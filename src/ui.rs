use crate::config::ThemeColors;
use crate::model::{AppState, PopupState, Quote};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, Clear, Padding, Paragraph, Row, Table, Tabs},
};

pub fn draw(frame: &mut Frame, state: &AppState, theme: &ThemeColors) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .horizontal_margin(2)
        .vertical_margin(1)
        .constraints([
            Constraint::Length(1), // Title
            Constraint::Length(1), // Thick rule
            Constraint::Length(1), // Group tabs
            Constraint::Length(1), // Spacing
            Constraint::Min(0),    // Table
            Constraint::Length(1), // Thin rule
            Constraint::Length(1), // Footer
        ])
        .split(area);

    draw_title(frame, chunks[0], state, theme);

    frame.render_widget(
        Block::default()
            .borders(Borders::TOP)
            .border_type(BorderType::Thick)
            .border_style(Style::new().fg(theme.fg)),
        chunks[1],
    );

    let tabs = Tabs::new(state.groups.iter().map(|g| g.group.as_str()))
        .select(state.current_tab)
        .style(Style::new().fg(theme.fg).add_modifier(Modifier::DIM))
        .highlight_style(
            Style::new()
                .fg(theme.selected)
                .add_modifier(Modifier::BOLD)
                .remove_modifier(Modifier::DIM),
        )
        .divider("")
        .padding("", "   ");
    frame.render_widget(tabs, chunks[2]);

    draw_table(frame, chunks[4], state, theme);

    frame.render_widget(
        Block::default()
            .borders(Borders::TOP)
            .border_style(Style::new().fg(theme.border)),
        chunks[5],
    );

    draw_status(frame, chunks[6], state, theme);

    // Draw popup on top if active
    if !matches!(state.popup, PopupState::None) {
        draw_popup(frame, area, state, theme);
    }
}

fn draw_table(frame: &mut Frame, area: Rect, state: &AppState, theme: &ThemeColors) {
    let Some(group) = state.current_group() else {
        let empty =
            Paragraph::new("No data").style(Style::new().fg(theme.fg).add_modifier(Modifier::DIM));
        frame.render_widget(empty, area);
        return;
    };

    let header = Row::new(vec![
        Cell::from("代碼"),
        Cell::from("名稱"),
        Cell::from(Line::from("現價").alignment(Alignment::Right)),
        Cell::from(Line::from("漲跌").alignment(Alignment::Right)),
        Cell::from(Line::from("%").alignment(Alignment::Right)),
        Cell::from(Line::from("最高").alignment(Alignment::Right)),
        Cell::from(Line::from("最低").alignment(Alignment::Right)),
        Cell::from(Line::from("成交量").alignment(Alignment::Right)),
    ])
    .style(header_style(theme))
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

    let table = Table::new(rows, widths)
        .header(header)
        .row_highlight_style(
            Style::new()
                .bg(theme.selected)
                .fg(theme.bg)
                .add_modifier(Modifier::BOLD),
        )
        .column_spacing(2);

    // Render as stateful widget with selection
    let mut table_state = ratatui::widgets::TableState::default();
    table_state.select(Some(state.selected_index));
    frame.render_stateful_widget(table, area, &mut table_state);
}

fn draw_title(frame: &mut Frame, area: Rect, state: &AppState, theme: &ThemeColors) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(10), Constraint::Min(0)])
        .split(area);

    let title =
        Paragraph::new("台股行情").style(Style::new().fg(theme.fg).add_modifier(Modifier::BOLD));
    frame.render_widget(title, chunks[0]);

    let index = &state.index_quote;
    let dim = Style::new().fg(theme.fg).add_modifier(Modifier::DIM);
    let index_text = if index.is_valid() {
        let change_color = if index.change > 0.0 {
            theme.up
        } else if index.change < 0.0 {
            theme.down
        } else {
            theme.fg
        };

        Line::from(vec![
            Span::styled(index.name.clone(), dim),
            Span::raw("  "),
            Span::styled(
                index.price_display(),
                Style::new().fg(change_color).add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(index.change_display(), Style::new().fg(change_color)),
            Span::raw("  "),
            Span::styled(index.pct_display(), Style::new().fg(change_color)),
        ])
    } else {
        Line::from(Span::styled("大盤指數 --", dim))
    };

    frame.render_widget(
        Paragraph::new(index_text).alignment(Alignment::Right),
        chunks[1],
    );
}

fn header_style(theme: &ThemeColors) -> Style {
    Style::new()
        .fg(theme.fg)
        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
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

    let right = |text: String| Line::from(text).alignment(Alignment::Right);

    Row::new(vec![
        Cell::from(q.code.as_str()).style(row_style),
        Cell::from(q.name.as_str()).style(row_style),
        Cell::from(right(q.price_display())).style(
            Style::new()
                .fg(price_color)
                .add_modifier(Modifier::BOLD)
                .bg(row_style.bg.unwrap_or(theme.bg)),
        ),
        Cell::from(right(q.change_display())).style(
            Style::new()
                .fg(change_color)
                .bg(row_style.bg.unwrap_or(theme.bg)),
        ),
        Cell::from(right(q.pct_display())).style(
            Style::new()
                .fg(change_color)
                .bg(row_style.bg.unwrap_or(theme.bg)),
        ),
        Cell::from(right(q.high_display())).style(
            Style::new()
                .fg(range_color(q.high))
                .bg(row_style.bg.unwrap_or(theme.bg)),
        ),
        Cell::from(right(q.low_display())).style(
            Style::new()
                .fg(range_color(q.low))
                .bg(row_style.bg.unwrap_or(theme.bg)),
        ),
        Cell::from(right(q.volume_display())).style(
            Style::new()
                .fg(theme.fg)
                .bg(row_style.bg.unwrap_or(theme.bg)),
        ),
    ])
    .height(1)
    .style(row_style)
}

fn draw_status(frame: &mut Frame, area: Rect, state: &AppState, theme: &ThemeColors) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(if state.loading { 18 } else { 10 }),
        ])
        .split(area);

    let key = Style::new().fg(theme.fg).add_modifier(Modifier::BOLD);
    let label = Style::new().fg(theme.fg).add_modifier(Modifier::DIM);

    let hints = [("←→", "分組"), ("↑↓", "選取"), ("h", "說明")];
    let spans: Vec<Span> = hints
        .iter()
        .flat_map(|(k, l)| {
            [
                Span::styled(format!("{} ", k), key),
                Span::styled(format!("{}  ", l), label),
            ]
        })
        .collect();
    frame.render_widget(Paragraph::new(Line::from(spans)), chunks[0]);

    let mut time_spans = Vec::new();
    if state.loading {
        time_spans.push(Span::styled(
            "更新中  ",
            Style::new().fg(theme.selected).add_modifier(Modifier::BOLD),
        ));
    }
    time_spans.push(Span::styled(
        state.last_update.format("%H:%M:%S").to_string(),
        label,
    ));
    frame.render_widget(
        Paragraph::new(Line::from(time_spans)).alignment(Alignment::Right),
        chunks[1],
    );
}

fn draw_popup(frame: &mut Frame, area: Rect, state: &AppState, theme: &ThemeColors) {
    let popup_area = if matches!(state.popup, PopupState::Help) {
        let width = 48.min(area.width);
        let height = 20.min(area.height);
        Rect::new(
            area.x + (area.width - width) / 2,
            area.y + (area.height - height) / 2,
            width,
            height,
        )
    } else {
        centered_rect(60, 40, area)
    };

    // Clear background using Clear widget
    frame.render_widget(Clear, popup_area);

    match &state.popup {
        PopupState::AddStock { group, code, error } => {
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
        PopupState::Help => draw_help_popup(frame, popup_area, theme),
        PopupState::None => {}
    }
}

fn popup_block(title: String, accent: Color, theme: &ThemeColors) -> Block<'static> {
    Block::default()
        .title(title)
        .title_style(Style::new().fg(accent).add_modifier(Modifier::BOLD))
        .borders(Borders::TOP)
        .border_type(BorderType::Thick)
        .border_style(Style::new().fg(accent))
        .padding(Padding::new(1, 1, 1, 0))
        .style(Style::new().bg(theme.bg).fg(theme.fg))
}

fn hint_style(theme: &ThemeColors) -> Style {
    Style::new().fg(theme.fg).add_modifier(Modifier::DIM)
}

fn draw_add_stock_popup(
    frame: &mut Frame,
    area: Rect,
    theme: &ThemeColors,
    group: &str,
    code: &str,
    error: &Option<String>,
) {
    let block = popup_block(format!("新增股票至 {}", group), theme.selected, theme);

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
    let code_text = format!("► 代碼  {}", code);
    let code_para = Paragraph::new(code_text)
        .style(Style::new().fg(theme.selected).add_modifier(Modifier::BOLD));
    frame.render_widget(code_para, chunks[0]);

    // Error message
    if let Some(err) = error {
        let err_para = Paragraph::new(format!("  ✗ {}", err))
            .style(Style::new().fg(theme.up).add_modifier(Modifier::BOLD));
        frame.render_widget(err_para, chunks[1]);
    }

    // Hint
    let hint = Paragraph::new("輸入股票代碼  Enter 確認  Esc 取消").style(hint_style(theme));
    frame.render_widget(hint, chunks[3]);
}

fn draw_add_group_popup(frame: &mut Frame, area: Rect, theme: &ThemeColors, name: &str) {
    let block = popup_block("新增分組".into(), theme.selected, theme);

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

    let name_text = format!("► 名稱  {}", name);
    let name_para = Paragraph::new(name_text)
        .style(Style::new().fg(theme.selected).add_modifier(Modifier::BOLD));
    frame.render_widget(name_para, chunks[0]);

    let hint = Paragraph::new("Enter 確認  Esc 取消").style(hint_style(theme));
    frame.render_widget(hint, chunks[2]);
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
    let block = popup_block(format!("編輯股票 {}", group), theme.selected, theme);

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
        "► 代碼  "
    } else {
        "  代碼  "
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
        "► 名稱  "
    } else {
        "  名稱  "
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
    let hint =
        Paragraph::new("Tab/Shift+Tab 切換欄位  Enter 確認  Esc 取消").style(hint_style(theme));
    frame.render_widget(hint, chunks[3]);
}

fn draw_delete_confirm_popup(frame: &mut Frame, area: Rect, theme: &ThemeColors, name: &str) {
    let block = popup_block("確認刪除".into(), theme.up, theme);

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

    let msg = Paragraph::new(format!("確定要刪除「{}」嗎？", name))
        .style(Style::new().fg(theme.fg).add_modifier(Modifier::BOLD));
    frame.render_widget(msg, chunks[0]);

    let hint = Paragraph::new(Line::from(vec![
        Span::styled("y ", Style::new().fg(theme.up).add_modifier(Modifier::BOLD)),
        Span::styled("確認刪除  ", hint_style(theme)),
        Span::styled(
            "n/Esc ",
            Style::new().fg(theme.fg).add_modifier(Modifier::BOLD),
        ),
        Span::styled("取消", hint_style(theme)),
    ]));
    frame.render_widget(hint, chunks[2]);
}

fn draw_help_popup(frame: &mut Frame, area: Rect, theme: &ThemeColors) {
    let block = popup_block("快捷鍵".into(), theme.selected, theme);

    let section = Style::new().fg(theme.fg).add_modifier(Modifier::BOLD);
    let key = Style::new().fg(theme.selected).add_modifier(Modifier::BOLD);

    let sections: [(&str, &[(&str, &str)]); 3] = [
        ("導航", &[("←/→ Tab", "切換分組"), ("↑/↓", "選取股票")]),
        (
            "操作",
            &[
                ("a", "在目前分組新增股票"),
                ("A", "新增分組"),
                ("e", "編輯選取的股票"),
                ("d", "刪除選取的股票(空分組則刪除分組)"),
            ],
        ),
        (
            "系統",
            &[
                ("r", "立即重抓報價"),
                ("t", "切換色彩主題"),
                ("h", "開啟／關閉說明"),
                ("q/Esc", "離開"),
            ],
        ),
    ];

    let mut lines = Vec::new();
    for (i, (title, items)) in sections.iter().enumerate() {
        if i > 0 {
            lines.push(Line::default());
        }
        lines.push(Line::from(Span::styled(*title, section)));
        for (k, desc) in items.iter() {
            lines.push(Line::from(vec![
                Span::styled(format!("  {:<10}", k), key),
                Span::styled(*desc, Style::new().fg(theme.fg)),
            ]));
        }
    }
    lines.push(Line::default());
    lines.push(Line::from(Span::styled("h/Esc 關閉", hint_style(theme))));

    frame.render_widget(Paragraph::new(lines).block(block), area);
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
