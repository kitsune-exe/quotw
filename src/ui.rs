use crate::config::ThemeColors;
use crate::model::{AppState, PopupState, Quote};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table},
};

pub fn draw(frame: &mut Frame, state: &AppState, theme: &ThemeColors) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(0),    // Table
            Constraint::Length(6), // Status bar (merged with key hints)
        ])
        .split(area);

    draw_title(frame, chunks[0], state, theme);
    draw_table(frame, chunks[1], state, theme);
    draw_status(frame, chunks[2], state, theme);

    // Draw popup on top if active
    if !matches!(state.popup, PopupState::None) {
        draw_popup(frame, area, state, theme);
    }
}

fn draw_table(frame: &mut Frame, area: Rect, state: &AppState, theme: &ThemeColors) {
    let Some(group) = state.current_group() else {
        let empty = Paragraph::new("No data").alignment(Alignment::Center);
        frame.render_widget(empty, area);
        return;
    };

    let header = Row::new(vec![
        Cell::from("代碼").style(header_style(theme)),
        Cell::from("名稱").style(header_style(theme)),
        Cell::from("現價").style(header_style(theme)),
        Cell::from("漲跌").style(header_style(theme)),
        Cell::from("%").style(header_style(theme)),
        Cell::from("最高").style(header_style(theme)),
        Cell::from("最低").style(header_style(theme)),
        Cell::from("成交量").style(header_style(theme)),
    ])
    .height(1);

    let rows: Vec<Row> = group.items.iter().map(|q| quote_to_row(q, theme)).collect();

    let widths = [
        Constraint::Length(8),  // 代碼
        Constraint::Length(12), // 名稱
        Constraint::Length(10), // 現價
        Constraint::Length(10), // 漲跌
        Constraint::Length(8),  // %
        Constraint::Length(10), // 最高
        Constraint::Length(10), // 最低
        Constraint::Length(10), // 成交量
    ];

    let block = Block::default()
        .title(format!(" {} ", group.group))
        .title_style(Style::new().fg(theme.selected).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::new().fg(theme.border));

    let table = Table::new(rows, widths)
        .header(header)
        .block(block)
        .row_highlight_style(
            Style::new()
                .bg(theme.selected)
                .fg(theme.bg)
                .add_modifier(Modifier::BOLD),
        )
        .column_spacing(1);

    // Render as stateful widget with selection
    let mut table_state = ratatui::widgets::TableState::default();
    table_state.select(Some(state.selected_index));
    frame.render_stateful_widget(table, area, &mut table_state);
}

fn draw_title(frame: &mut Frame, area: Rect, state: &AppState, theme: &ThemeColors) {
    let index = &state.index_quote;
    let title_text = if index.is_valid() {
        let (price_color, change_color) = if index.change > 0.0 {
            (theme.up, theme.up)
        } else if index.change < 0.0 {
            (theme.down, theme.down)
        } else {
            (theme.fg, theme.fg)
        };

        Line::from(vec![
            Span::styled(
                "📈 台灣股市即時行情  ",
                Style::new()
                    .fg(theme.header_bg)
                    .bg(theme.header_fg)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                index.name.clone(),
                Style::new()
                    .fg(theme.header_bg)
                    .bg(theme.header_fg)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" ", Style::new().fg(theme.header_bg).bg(theme.header_fg)),
            Span::styled(
                index.price_display(),
                Style::new()
                    .fg(price_color)
                    .bg(theme.header_fg)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" ", Style::new().fg(theme.header_bg).bg(theme.header_fg)),
            Span::styled(
                index.change_display(),
                Style::new()
                    .fg(change_color)
                    .bg(theme.header_fg)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" ", Style::new().fg(theme.header_bg).bg(theme.header_fg)),
            Span::styled(
                index.pct_display(),
                Style::new()
                    .fg(change_color)
                    .bg(theme.header_fg)
                    .add_modifier(Modifier::BOLD),
            ),
        ])
    } else {
        Line::from(vec![Span::styled(
            "📈 台灣股市即時行情  大盤指數 --",
            Style::new()
                .fg(theme.header_bg)
                .bg(theme.header_fg)
                .add_modifier(Modifier::BOLD),
        )])
    };

    let title = Paragraph::new(title_text)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::new().fg(theme.border))
                .style(Style::new().bg(theme.header_fg)),
        );
    frame.render_widget(title, area);
}

fn header_style(theme: &ThemeColors) -> Style {
    Style::new()
        .fg(theme.header_fg)
        .bg(theme.header_bg)
        .add_modifier(Modifier::BOLD)
}

fn quote_to_row<'a>(q: &'a Quote, theme: &'a ThemeColors) -> Row<'a> {
    let is_limit_up = q.is_limit_up();
    let is_limit_down = q.is_limit_down();

    let (price_color, change_color, row_style) = if !q.is_valid() {
        (theme.fg, theme.fg, Style::new().fg(theme.fg))
    } else if is_limit_up {
        // 漲停：紅底白字
        (
            Color::White,
            Color::White,
            Style::new().fg(Color::White).bg(theme.up),
        )
    } else if is_limit_down {
        // 跌停：綠底白字
        (
            Color::White,
            Color::White,
            Style::new().fg(Color::White).bg(theme.down),
        )
    } else if q.change > 0.0 {
        (theme.up, theme.up, Style::new().fg(theme.fg))
    } else if q.change < 0.0 {
        (theme.down, theme.down, Style::new().fg(theme.fg))
    } else {
        (theme.fg, theme.fg, Style::new().fg(theme.fg))
    };

    // 最高/最低依與昨收比較著色；漲跌停列沿用白字
    let range_color = |v: f64| {
        if is_limit_up || is_limit_down {
            Color::White
        } else if !v.is_finite() || !q.prev_close.is_finite() {
            theme.fg
        } else if v > q.prev_close {
            theme.up
        } else if v < q.prev_close {
            theme.down
        } else {
            theme.fg
        }
    };

    Row::new(vec![
        Cell::from(q.code.as_str()).style(row_style),
        Cell::from(q.name.as_str()).style(row_style),
        Cell::from(q.price_display()).style(
            Style::new()
                .fg(price_color)
                .add_modifier(Modifier::BOLD)
                .bg(row_style.bg.unwrap_or(theme.bg)),
        ),
        Cell::from(q.change_display()).style(
            Style::new()
                .fg(change_color)
                .bg(row_style.bg.unwrap_or(theme.bg)),
        ),
        Cell::from(q.pct_display()).style(
            Style::new()
                .fg(change_color)
                .bg(row_style.bg.unwrap_or(theme.bg)),
        ),
        Cell::from(q.high_display()).style(
            Style::new()
                .fg(range_color(q.high))
                .bg(row_style.bg.unwrap_or(theme.bg)),
        ),
        Cell::from(q.low_display()).style(
            Style::new()
                .fg(range_color(q.low))
                .bg(row_style.bg.unwrap_or(theme.bg)),
        ),
        Cell::from(q.volume_display()).style(
            Style::new()
                .fg(theme.fg)
                .bg(row_style.bg.unwrap_or(theme.bg)),
        ),
    ])
    .height(1)
    .style(row_style)
}

fn draw_status(frame: &mut Frame, area: Rect, state: &AppState, theme: &ThemeColors) {
    let loading_indicator = if state.loading { " 🔄" } else { "" };

    let status = Paragraph::new(vec![
        Line::from(vec![
            Span::styled(
                format!(" 更新: {} ", state.last_update.format("%H:%M:%S")),
                Style::new().fg(theme.fg).bg(theme.bg),
            ),
            Span::styled(
                loading_indicator,
                Style::new()
                    .fg(theme.selected)
                    .bg(theme.bg)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                " 導航: ",
                Style::new()
                    .fg(theme.header_bg)
                    .bg(theme.header_fg)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "←/→ ",
                Style::new().fg(theme.selected).add_modifier(Modifier::BOLD),
            ),
            Span::styled("切換分頁  ", Style::new().fg(theme.fg)),
            Span::styled(
                "↑/↓ ",
                Style::new().fg(theme.selected).add_modifier(Modifier::BOLD),
            ),
            Span::styled("選取股票", Style::new().fg(theme.fg)),
        ]),
        Line::from(vec![
            Span::styled(
                " 操作: ",
                Style::new()
                    .fg(theme.header_bg)
                    .bg(theme.header_fg)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "a ",
                Style::new().fg(theme.selected).add_modifier(Modifier::BOLD),
            ),
            Span::styled("新增股票  ", Style::new().fg(theme.fg)),
            Span::styled(
                "A ",
                Style::new().fg(theme.selected).add_modifier(Modifier::BOLD),
            ),
            Span::styled("新增分組  ", Style::new().fg(theme.fg)),
            Span::styled(
                "e ",
                Style::new().fg(theme.selected).add_modifier(Modifier::BOLD),
            ),
            Span::styled("編輯  ", Style::new().fg(theme.fg)),
            Span::styled(
                "d ",
                Style::new().fg(theme.selected).add_modifier(Modifier::BOLD),
            ),
            Span::styled("刪除", Style::new().fg(theme.fg)),
        ]),
        Line::from(vec![
            Span::styled(
                " 系統: ",
                Style::new()
                    .fg(theme.header_bg)
                    .bg(theme.header_fg)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "r ",
                Style::new().fg(theme.selected).add_modifier(Modifier::BOLD),
            ),
            Span::styled("立即重抓  ", Style::new().fg(theme.fg)),
            Span::styled(
                "t ",
                Style::new().fg(theme.selected).add_modifier(Modifier::BOLD),
            ),
            Span::styled("切換主題  ", Style::new().fg(theme.fg)),
            Span::styled(
                "q/Esc ",
                Style::new().fg(theme.selected).add_modifier(Modifier::BOLD),
            ),
            Span::styled("離開", Style::new().fg(theme.fg)),
        ]),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::new().fg(theme.border))
            .title(" 快捷鍵 ")
            .title_style(Style::new().fg(theme.selected).add_modifier(Modifier::BOLD)),
    );

    frame.render_widget(status, area);
}

fn draw_popup(frame: &mut Frame, area: Rect, state: &AppState, theme: &ThemeColors) {
    let popup_area = centered_rect(60, 40, area);

    // Clear background using Clear widget
    frame.render_widget(Clear, popup_area);

    match &state.popup {
        PopupState::AddStock {
            group,
            code,
            name: _,
            field: _,
            error,
        } => {
            draw_add_stock_popup(frame, popup_area, theme, group, code, error);
        }
        PopupState::AddGroup { name } => {
            draw_add_group_popup(frame, popup_area, theme, name);
        }
        PopupState::EditStock {
            group,
            index: _,
            code,
            name,
            field,
        } => {
            draw_edit_stock_popup(frame, popup_area, theme, group, code, name, *field);
        }
        PopupState::DeleteConfirm { item_type: _, name } => {
            draw_delete_confirm_popup(frame, popup_area, theme, name);
        }
        PopupState::None => {}
    }
}

fn draw_add_stock_popup(
    frame: &mut Frame,
    area: Rect,
    theme: &ThemeColors,
    group: &str,
    code: &str,
    error: &Option<String>,
) {
    let block = Block::default()
        .title(format!(" 新增股票至 [{}] ", group))
        .title_style(Style::new().fg(theme.selected).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::new().fg(theme.selected))
        .style(Style::new().bg(theme.bg).fg(theme.fg));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(inner);

    // Code input
    let code_text = format!("► 代碼: {}", code);
    let code_para = Paragraph::new(code_text)
        .style(Style::new().fg(theme.selected).add_modifier(Modifier::BOLD));
    frame.render_widget(code_para, chunks[0]);

    // Error message
    if let Some(err) = error {
        let err_para = Paragraph::new(format!("  ✗ {}", err))
            .style(Style::new().fg(theme.up).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);
        frame.render_widget(err_para, chunks[1]);
    }

    // Hint
    let hint = Paragraph::new("輸入股票代碼  Enter: 確認  Esc: 取消")
        .style(Style::new().fg(theme.fg))
        .alignment(Alignment::Center);
    frame.render_widget(hint, chunks[3]);
}

fn draw_add_group_popup(frame: &mut Frame, area: Rect, theme: &ThemeColors, name: &str) {
    let block = Block::default()
        .title(" 新增分組 ")
        .title_style(Style::new().fg(theme.selected).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::new().fg(theme.selected))
        .style(Style::new().bg(theme.bg).fg(theme.fg));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(inner);

    let name_text = format!("  分組名稱: {}", name);
    let name_para = Paragraph::new(name_text)
        .style(Style::new().fg(theme.selected).add_modifier(Modifier::BOLD));
    frame.render_widget(name_para, chunks[0]);

    let hint = Paragraph::new("Enter: 確認  Esc: 取消")
        .style(Style::new().fg(theme.fg))
        .alignment(Alignment::Center);
    frame.render_widget(hint, chunks[1]);
}

fn draw_edit_stock_popup(
    frame: &mut Frame,
    area: Rect,
    theme: &ThemeColors,
    group: &str,
    code: &str,
    name: &str,
    field: usize,
) {
    let block = Block::default()
        .title(format!(" 編輯股票 [{}] ", group))
        .title_style(Style::new().fg(theme.selected).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::new().fg(theme.selected))
        .style(Style::new().bg(theme.bg).fg(theme.fg));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(inner);

    // Code input
    let code_label = if field == 0 {
        "► 代碼: "
    } else {
        "  代碼: "
    };
    let code_text = format!("{}{}", code_label, code);
    let code_style = if field == 0 {
        Style::new().fg(theme.selected).add_modifier(Modifier::BOLD)
    } else {
        Style::new().fg(theme.fg)
    };
    let code_para = Paragraph::new(code_text).style(code_style);
    frame.render_widget(code_para, chunks[0]);

    // Name input
    let name_label = if field == 1 {
        "► 名稱: "
    } else {
        "  名稱: "
    };
    let name_text = format!("{}{}", name_label, name);
    let name_style = if field == 1 {
        Style::new().fg(theme.selected).add_modifier(Modifier::BOLD)
    } else {
        Style::new().fg(theme.fg)
    };
    let name_para = Paragraph::new(name_text).style(name_style);
    frame.render_widget(name_para, chunks[1]);

    // Hint
    let hint = Paragraph::new("Tab/Shift+Tab: 切換欄位  Enter: 確認  Esc: 取消")
        .style(Style::new().fg(theme.fg))
        .alignment(Alignment::Center);
    frame.render_widget(hint, chunks[3]);
}

fn draw_delete_confirm_popup(frame: &mut Frame, area: Rect, theme: &ThemeColors, name: &str) {
    let block = Block::default()
        .title(" 確認刪除 ")
        .title_style(Style::new().fg(theme.up).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::new().fg(theme.up))
        .style(Style::new().bg(theme.bg).fg(theme.fg));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(inner);

    let msg = Paragraph::new(format!("  確定要刪除 \"{}\" 嗎？", name))
        .style(Style::new().fg(theme.fg))
        .alignment(Alignment::Center);
    frame.render_widget(msg, chunks[0]);

    let hint = Paragraph::new("  y: 確認刪除  n/Esc: 取消")
        .style(Style::new().fg(theme.up).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);
    frame.render_widget(hint, chunks[1]);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
