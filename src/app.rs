use crate::api::{fetch_index, fetch_quotes};
use crate::config::{load_portfolio, save_config, save_portfolio, Config, Portfolio, ThemeColors, WatchItem};
use crate::model::{AppState, DeleteType, GroupView, IndexQuote, PopupState, Quote};
use crate::ui::draw;
use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::DefaultTerminal;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::interval;
use tokio::sync::mpsc;

pub async fn run(terminal: &mut DefaultTerminal, config: Config, theme: ThemeColors) -> Result<()> {
    let refresh_interval = Duration::from_secs(config.refresh_interval_secs);

    // Load portfolio and build watchlist from it
    let mut portfolio = load_portfolio()?.unwrap_or_default();
    let watchlist = build_watchlist_from_portfolio(&portfolio);

    // Initial fetch
    let quotes = fetch_initial_quotes(&watchlist).await?;
    let index = fetch_index().await.unwrap_or_else(|_| IndexQuote::empty());
    let groups = build_groups(&watchlist, quotes, &portfolio);
    let mut state = AppState::new(groups, config.theme.clone(), config.refresh_interval_secs);
    state.index_quote = index;

    let mut current_theme = theme;
    let theme_names = ["catppuccin_mocha", "monokai_classic", "tokyo_night"];
    let mut theme_idx = theme_names
        .iter()
        .position(|&n| n == config.theme)
        .unwrap_or(0);

    // Channel for crossterm events
    let (tx, mut rx) = mpsc::unbounded_channel();
    std::thread::spawn(move || {
        loop {
            if let Ok(event) = event::read() {
                if tx.send(event).is_err() {
                    break;
                }
            }
        }
    });

    let mut tick_interval = interval(refresh_interval);
    // Skip the first immediate tick
    tick_interval.tick().await;

    // Helper to refresh quotes using the portfolio watchlist
    let refresh_quotes = async |watchlist: &[WatchItem]| -> Result<Vec<Quote>> {
        fetch_initial_quotes(watchlist).await
    };

    loop {
        terminal.draw(|frame| draw(frame, &state, &current_theme))?;

        tokio::select! {
            // Handle keyboard events
            Some(event) = rx.recv() => {
                if let Event::Key(key) = event {
                    if key.kind == KeyEventKind::Press {
                        // Handle popup input first
                        if !matches!(state.popup, PopupState::None) {
                            handle_popup_input(&key.code, &mut state, &mut portfolio).await?;
                            continue;
                        }

                        match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => break,
                            KeyCode::Tab | KeyCode::Right => state.next_tab(),
                            KeyCode::BackTab | KeyCode::Left => state.prev_tab(),
                            KeyCode::Char('r') => {
                                state.loading = true;
                                terminal.draw(|frame| draw(frame, &state, &current_theme))?;
                                let watchlist = build_watchlist_from_portfolio(&portfolio);
                                if let Ok(quotes) = refresh_quotes(&watchlist).await {
                                    let groups = build_groups(&watchlist, quotes, &portfolio);
                                    state.groups = groups;
                                    state.last_update = chrono::Local::now();
                                }
                                if let Ok(index) = fetch_index().await {
                                    state.index_quote = index;
                                }
                                state.loading = false;
                            }
                            KeyCode::Char('t') => {
                                theme_idx = (theme_idx + 1) % theme_names.len();
                                current_theme = ThemeColors::from_name(theme_names[theme_idx]);
                                state.theme_name = theme_names[theme_idx].into();
                                let mut new_config = config.clone();
                                new_config.theme = theme_names[theme_idx].into();
                                let _ = save_config(&new_config);
                            }
                            KeyCode::Char('a') => {
                                // Add stock to current group (only need code)
                                if let Some(group) = state.current_group() {
                                    state.popup = PopupState::AddStock {
                                        group: group.group.clone(),
                                        code: String::new(),
                                        name: String::new(),
                                        field: 0,
                                        error: None,
                                    };
                                }
                            }
                            KeyCode::Char('A') => {
                                // Add new group
                                state.popup = PopupState::AddGroup { name: String::new() };
                            }
                            KeyCode::Char('e') => {
                                // Edit selected stock
                                if let Some(stock) = state.selected_stock() {
                                    state.popup = PopupState::EditStock {
                                        group: state.current_group().unwrap().group.clone(),
                                        index: state.selected_index,
                                        code: stock.code.clone(),
                                        name: stock.name.clone(),
                                        field: 0,
                                    };
                                }
                            }
                            KeyCode::Char('d') => {
                                // Delete selected stock or group
                                if let Some(group) = state.current_group() {
                                    if !group.items.is_empty() {
                                        // Clamp index to valid range just in case
                                        let idx = state.selected_index.min(group.items.len().saturating_sub(1));
                                        state.popup = PopupState::DeleteConfirm {
                                            item_type: DeleteType::Stock {
                                                group: group.group.clone(),
                                                index: idx,
                                            },
                                            name: format!("{} ({})", group.items[idx].name, group.items[idx].code),
                                        };
                                    } else {
                                        // Empty group - offer to delete group
                                        state.popup = PopupState::DeleteConfirm {
                                            item_type: DeleteType::Group {
                                                group: group.group.clone(),
                                            },
                                            name: group.group.clone(),
                                        };
                                    }
                                }
                            }
                            KeyCode::Up => state.select_prev(),
                            KeyCode::Down => state.select_next(),
                            _ => {}
                        }
                    }
                }
            }
            // Periodic update
            _ = tick_interval.tick() => {
                state.loading = true;
                terminal.draw(|frame| draw(frame, &state, &current_theme))?;
                let watchlist = build_watchlist_from_portfolio(&portfolio);
                if let Ok(quotes) = refresh_quotes(&watchlist).await {
                    let groups = build_groups(&watchlist, quotes, &portfolio);
                    state.groups = groups;
                    state.clamp_selected_index();
                    state.last_update = chrono::Local::now();
                }
                if let Ok(index) = fetch_index().await {
                    state.index_quote = index;
                }
                state.loading = false;
            }
        }
    }

    Ok(())
}

async fn handle_popup_input(
    key: &KeyCode,
    state: &mut AppState,
    portfolio: &mut Portfolio,
) -> Result<()> {
    match (&mut state.popup, key) {
        // Add Stock - only need code
        (PopupState::AddStock { group, code, name, field, error: _ }, KeyCode::Esc) => {
            state.popup = PopupState::None;
        }
        (PopupState::AddStock { group, code, name, field, error: _ }, KeyCode::Char(c)) => {
            if c.is_ascii_alphanumeric() {
                code.push(c.to_ascii_uppercase());
            }
        }
        (PopupState::AddStock { group, code, name, field, error: _ }, KeyCode::Backspace) => {
            code.pop();
        }
        (PopupState::AddStock { group, code, name, field: _, error }, KeyCode::Enter) => {
            if !code.is_empty() {
                // Clear previous error
                *error = None;
                // Validate code by fetching quote
                match fetch_quotes(&[code.clone()]).await {
                    Ok(quotes) => {
                        if let Some(quote) = quotes.first() {
                            if quote.is_valid() {
                                // Valid code - add to portfolio
                                if let Some(portfolio_group) = portfolio.groups.iter_mut().find(|g| g.group_name == *group) {
                                    portfolio_group.code.push(code.clone());
                                }
                                save_and_refresh(portfolio, state).await?;
                                state.popup = PopupState::None;
                            } else {
                                *error = Some("無效的股票代碼，查無資料".to_string());
                            }
                        } else {
                            *error = Some("無效的股票代碼，查無資料".to_string());
                        }
                    }
                    Err(_) => {
                        *error = Some("網路錯誤，請稍後再試".to_string());
                    }
                }
            }
        }

        // Add Group
        (PopupState::AddGroup { name }, KeyCode::Esc) => {
            state.popup = PopupState::None;
        }
        (PopupState::AddGroup { name }, KeyCode::Char(c)) => {
            name.push(*c);
        }
        (PopupState::AddGroup { name }, KeyCode::Backspace) => {
            name.pop();
        }
        (PopupState::AddGroup { name }, KeyCode::Enter) => {
            if !name.is_empty() {
                portfolio.groups.push(crate::config::PortfolioGroup {
                    group_name: name.clone(),
                    code: vec![],
                });
                save_and_refresh(portfolio, state).await?;
                state.popup = PopupState::None;
            }
        }

        // Edit Stock
        (PopupState::EditStock { group, index, code, name, field }, KeyCode::Esc) => {
            state.popup = PopupState::None;
        }
        (PopupState::EditStock { group, index, code, name, field }, KeyCode::Tab) => {
            *field = (*field + 1) % 2;
        }
        (PopupState::EditStock { group, index, code, name, field }, KeyCode::BackTab) => {
            *field = if *field == 0 { 1 } else { 0 };
        }
        (PopupState::EditStock { group, index, code, name, field }, KeyCode::Char(c)) => {
            if *field == 0 && c.is_ascii_alphanumeric() {
                code.push(c.to_ascii_uppercase());
            } else if *field == 1 {
                name.push(*c);
            }
        }
        (PopupState::EditStock { group, index, code, name, field }, KeyCode::Backspace) => {
            if *field == 0 && !code.is_empty() {
                code.pop();
            } else if *field == 1 && !name.is_empty() {
                name.pop();
            }
        }
        (PopupState::EditStock { group, index, code, name, field: _ }, KeyCode::Enter) => {
            if !code.is_empty() && !name.is_empty() {
                if let Some(portfolio_group) = portfolio.groups.iter_mut().find(|g| g.group_name == *group) {
                    if *index < portfolio_group.code.len() {
                        portfolio_group.code[*index] = code.clone();
                    }
                }
                save_and_refresh(portfolio, state).await?;
                state.popup = PopupState::None;
            }
        }

        // Delete Confirm
        (PopupState::DeleteConfirm { item_type, name }, KeyCode::Esc) | (PopupState::DeleteConfirm { item_type, name }, KeyCode::Char('n')) => {
            state.popup = PopupState::None;
        }
        (PopupState::DeleteConfirm { item_type, name }, KeyCode::Char('y')) => {
            match item_type {
                DeleteType::Stock { group, index } => {
                    if let Some(portfolio_group) = portfolio.groups.iter_mut().find(|g| g.group_name == *group) {
                        if *index < portfolio_group.code.len() {
                            portfolio_group.code.remove(*index);
                        }
                    }
                }
                DeleteType::Group { group } => {
                    portfolio.groups.retain(|g| g.group_name != *group);
                }
            }
            save_and_refresh(portfolio, state).await?;
            state.popup = PopupState::None;
        }
        _ => {}
    }
    Ok(())
}

async fn save_and_refresh(portfolio: &Portfolio, state: &mut AppState) -> Result<()> {
    save_portfolio(portfolio)?;
    let watchlist = build_watchlist_from_portfolio(portfolio);
    let quotes = fetch_initial_quotes(&watchlist).await?;
    let groups = build_groups(&watchlist, quotes, portfolio);
    state.groups = groups;
    state.clamp_selected_index();
    state.last_update = chrono::Local::now();
    Ok(())
}

async fn fetch_initial_quotes(watchlist: &[WatchItem]) -> Result<Vec<Quote>> {
    let codes: Vec<String> = watchlist.iter().map(|w| w.code.clone()).collect();
    let quotes = fetch_quotes(&codes).await?;
    Ok(quotes)
}

fn build_groups(watchlist: &[WatchItem], quotes: Vec<Quote>, portfolio: &Portfolio) -> Vec<GroupView> {
    // Build quote map for quick lookup
    let quote_map: HashMap<String, Quote> = quotes.into_iter().map(|q| (q.code.clone(), q)).collect();

    // Group by group name, preserving order from portfolio
    let mut group_order: Vec<String> = Vec::new();
    let mut group_items: HashMap<String, Vec<Quote>> = HashMap::new();

    // First, add groups from watchlist (which comes from portfolio with codes)
    for item in watchlist {
        if !group_order.contains(&item.group) {
            group_order.push(item.group.clone());
        }

        let quote = quote_map
            .get(&item.code)
            .cloned()
            .unwrap_or_else(|| Quote::empty(&item.code, &item.name));

        group_items.entry(item.group.clone()).or_default().push(quote);
    }

    // Also include empty groups from portfolio that have no codes
    for pg in &portfolio.groups {
        if !group_order.contains(&pg.group_name) {
            group_order.push(pg.group_name.clone());
            // Empty group - no items
        }
    }

    group_order
        .into_iter()
        .map(|group| GroupView {
            group: group.clone(),
            items: group_items.remove(&group).unwrap_or_default(),
        })
        .collect()
}

fn build_watchlist_from_portfolio(portfolio: &Portfolio) -> Vec<WatchItem> {
    let mut watchlist = Vec::new();
    
    // Use groups from portfolio
    for group in &portfolio.groups {
        for code in &group.code {
            watchlist.push(WatchItem {
                code: code.clone(),
                name: code.clone(), // Name will be updated from API response
                group: group.group_name.clone(),
            });
        }
    }
    
    watchlist
}