use crate::api::fetch_quotes;
use crate::config::{load_portfolio, save_config, Config, Portfolio, ThemeColors, WatchItem};
use crate::model::{AppState, GroupView, Quote};
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
    let portfolio = load_portfolio()?.unwrap_or_default();
    let watchlist = build_watchlist_from_portfolio(&portfolio, &config.watchlist);

    // Initial fetch
    let quotes = fetch_initial_quotes(&watchlist).await?;
    let groups = build_groups(&watchlist, quotes);
    let mut state = AppState::new(groups, config.theme.clone(), config.refresh_interval_secs);

    let mut current_theme = theme;
    let theme_names = ["default", "dark", "high_contrast"];
    let mut theme_idx = theme_names
        .iter()
        .position(|&n| n == config.theme)
        .unwrap_or(1);

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
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => break,
                            KeyCode::Tab | KeyCode::Right => state.next_tab(),
                            KeyCode::BackTab | KeyCode::Left => state.prev_tab(),
                            KeyCode::Char('r') => {
                                state.loading = true;
                                terminal.draw(|frame| draw(frame, &state, &current_theme))?;
                                if let Ok(quotes) = refresh_quotes(&watchlist).await {
                                    let groups = build_groups(&watchlist, quotes);
                                    state.groups = groups;
                                    state.last_update = chrono::Local::now();
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
                            _ => {}
                        }
                    }
                }
            }
            // Periodic update
            _ = tick_interval.tick() => {
                state.loading = true;
                terminal.draw(|frame| draw(frame, &state, &current_theme))?;
                if let Ok(quotes) = refresh_quotes(&watchlist).await {
                    let groups = build_groups(&watchlist, quotes);
                    state.groups = groups;
                    state.last_update = chrono::Local::now();
                }
                state.loading = false;
            }
        }
    }

    Ok(())
}

async fn fetch_initial_quotes(watchlist: &[WatchItem]) -> Result<Vec<Quote>> {
    let codes: Vec<String> = watchlist.iter().map(|w| w.code.clone()).collect();
    let quotes = fetch_quotes(&codes).await?;
    Ok(quotes)
}

fn build_groups(watchlist: &[WatchItem], quotes: Vec<Quote>) -> Vec<GroupView> {
    // Build quote map for quick lookup
    let quote_map: HashMap<String, Quote> = quotes.into_iter().map(|q| (q.code.clone(), q)).collect();

    // Group by group name, preserving order of first appearance
    let mut group_order: Vec<String> = Vec::new();
    let mut group_items: HashMap<String, Vec<Quote>> = HashMap::new();

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

    group_order
        .into_iter()
        .map(|group| GroupView {
            group: group.clone(),
            items: group_items.remove(&group).unwrap_or_default(),
        })
        .collect()
}

fn build_watchlist_from_portfolio(portfolio: &Portfolio, fallback_watchlist: &[WatchItem]) -> Vec<WatchItem> {
    let mut watchlist = Vec::new();
    
    // If portfolio has groups, use them
    if !portfolio.groups.is_empty() {
        for group in &portfolio.groups {
            for code in &group.code {
                // Try to find name from fallback watchlist
                let name = fallback_watchlist
                    .iter()
                    .find(|w| w.code == *code)
                    .map(|w| w.name.clone())
                    .unwrap_or_else(|| code.clone());
                
                watchlist.push(WatchItem {
                    code: code.clone(),
                    name,
                    group: group.group_name.clone(),
                });
            }
        }
    } else {
        // Fallback to config watchlist
        watchlist = fallback_watchlist.to_vec();
    }
    
    watchlist
}