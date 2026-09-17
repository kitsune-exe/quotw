use crate::config::ThemeColors;
use crate::model::{AppState, Quote};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Tabs},
    Frame,
};

pub fn draw(frame: &mut Frame, state: &AppState, theme: &ThemeColors) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),    // Tabs
            Constraint::Min(0),       // Table
            Constraint::Length(1),    // Status bar
        ])
        .split(area);

    draw_tabs(frame, chunks[0], state, theme);
    draw_table(frame, chunks[1], state, theme);
    draw_status(frame, chunks[2], state, theme);
}

fn draw_tabs(frame: &mut Frame, area: Rect, state: &AppState, theme: &ThemeColors) {
    let titles = state
        .groups
        .iter()
        .map(|g| g.group.as_str())
        .collect::<Vec<_>>();

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::BOTTOM).border_style(Style::new().fg(theme.border)))
        .select(state.current_tab)
        .style(Style::new().fg(theme.fg).bg(theme.bg))
        .highlight_style(
            Style::new()
                .fg(theme.selected)
                .bg(theme.bg)
                .add_modifier(Modifier::BOLD),
        )
        .divider(" ");

    frame.render_widget(tabs, area);
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
        Cell::from("成交量").style(header_style(theme)),
    ])
    .height(1);

    let rows: Vec<Row> = group
        .items
        .iter()
        .map(|q| quote_to_row(q, theme))
        .collect();

    let widths = [
        Constraint::Length(8),   // 代碼
        Constraint::Length(12),  // 名稱
        Constraint::Length(10),  // 現價
        Constraint::Length(10),  // 漲跌
        Constraint::Length(8),   // %
        Constraint::Length(10),  // 成交量
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default().borders(Borders::ALL).border_style(Style::new().fg(theme.border)))
        .row_highlight_style(Style::new().bg(theme.selected).fg(theme.bg))
        .column_spacing(1);

    frame.render_widget(table, area);
}

fn header_style(theme: &ThemeColors) -> Style {
    Style::new()
        .fg(theme.header_fg)
        .bg(theme.header_bg)
        .add_modifier(Modifier::BOLD)
}

fn quote_to_row<'a>(q: &'a Quote, theme: &'a ThemeColors) -> Row<'a> {
    let (price_color, change_color) = if !q.is_valid() {
        (theme.fg, theme.fg)
    } else if q.change > 0.0 {
        (theme.up, theme.up)
    } else if q.change < 0.0 {
        (theme.down, theme.down)
    } else {
        (theme.fg, theme.fg)
    };

    Row::new(vec![
        Cell::from(q.code.as_str()).style(Style::new().fg(theme.fg)),
        Cell::from(q.name.as_str()).style(Style::new().fg(theme.fg)),
        Cell::from(q.price_display()).style(Style::new().fg(price_color).add_modifier(Modifier::BOLD)),
        Cell::from(q.change_display()).style(Style::new().fg(change_color)),
        Cell::from(q.pct_display()).style(Style::new().fg(change_color)),
        Cell::from(q.volume_display()).style(Style::new().fg(theme.fg)),
    ])
    .height(1)
}

fn draw_status(frame: &mut Frame, area: Rect, state: &AppState, theme: &ThemeColors) {
    let status_text = format!(
        " 更新: {}  |  Tab/←→:切換分頁  r:立即重抓  t:切換主題  q:離開 ",
        state.last_update.format("%H:%M:%S")
    );

    let loading_indicator = if state.loading { " 🔄" } else { "" };

    let status = Paragraph::new(Line::from(vec![
        Span::styled(status_text, Style::new().fg(theme.fg).bg(theme.bg)),
        Span::styled(loading_indicator, Style::new().fg(theme.selected).bg(theme.bg).add_modifier(Modifier::BOLD)),
    ]))
    .alignment(Alignment::Left)
    .block(Block::default().borders(Borders::TOP).border_style(Style::new().fg(theme.border)));

    frame.render_widget(status, area);
}