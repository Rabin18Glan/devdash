use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Cell, Paragraph, Row, Scrollbar, ScrollbarOrientation,
        ScrollbarState, Table,
    },
};

use crate::app::App;
use crate::ui::theme::THEME;

pub fn render(frame: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(8),    // Ports Table
            Constraint::Length(5), // Selected Port Quick Inspector
        ])
        .split(area);

    let filtered = app.filtered_ports();

    // Table Header
    let header_cells = [
        "",
        "Proto",
        "Port",
        "Address",
        "PID",
        "Process",
        "CPU%",
        "Memory",
        "URL / Tag",
    ]
    .iter()
    .map(|h| {
        Cell::from(Span::styled(
            *h,
            Style::default()
                .fg(THEME.primary)
                .add_modifier(Modifier::BOLD),
        ))
    });
    let header = Row::new(header_cells)
        .style(Style::default().bg(Color::Rgb(26, 29, 44)))
        .height(1);

    // Rows
    let rows: Vec<Row> = filtered
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let is_selected = i == app.ports_selected;

            let arrow = if is_selected { "▶" } else { " " };
            let proto_style = if p.protocol == "TCP" {
                Style::default().fg(THEME.accent)
            } else {
                Style::default().fg(THEME.warning)
            };

            let port_str = format!(":{}", p.port);
            let pid_str = p
                .pid
                .map(|n| n.to_string())
                .unwrap_or_else(|| "-".to_string());
            let cpu_str = if p.cpu_usage > 0.0 {
                format!("{:.1}%", p.cpu_usage)
            } else {
                "-".to_string()
            };
            let mem_str = if p.memory_mb > 0.0 {
                format!("{:.1} MB", p.memory_mb)
            } else {
                "-".to_string()
            };

            let tag_str = if p.is_docker_proxy {
                "🐳 Docker Container".to_string()
            } else {
                p.url()
            };

            let row_style = if is_selected {
                Style::default()
                    .bg(THEME.highlight_bg)
                    .fg(THEME.text)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(THEME.text)
            };

            Row::new(vec![
                Cell::from(Span::styled(arrow, Style::default().fg(THEME.primary))),
                Cell::from(Span::styled(&p.protocol, proto_style)),
                Cell::from(Span::styled(
                    port_str,
                    Style::default()
                        .fg(THEME.warning)
                        .add_modifier(Modifier::BOLD),
                )),
                Cell::from(Span::styled(
                    &p.address,
                    Style::default().fg(THEME.text_dim),
                )),
                Cell::from(Span::styled(pid_str, Style::default().fg(THEME.secondary))),
                Cell::from(Span::styled(
                    &p.process_name,
                    Style::default().fg(THEME.text),
                )),
                Cell::from(Span::styled(cpu_str, Style::default().fg(THEME.accent))),
                Cell::from(Span::styled(mem_str, Style::default().fg(THEME.text_dim))),
                Cell::from(Span::styled(tag_str, Style::default().fg(THEME.primary))),
            ])
            .style(row_style)
            .height(1)
        })
        .collect();

    let widths = [
        Constraint::Length(2),  // Arrow
        Constraint::Length(7),  // Proto
        Constraint::Length(9),  // Port
        Constraint::Length(18), // Address
        Constraint::Length(9),  // PID
        Constraint::Length(20), // Process
        Constraint::Length(8),  // CPU%
        Constraint::Length(12), // Mem
        Constraint::Min(24),    // URL / Tag
    ];

    let table_title = format!(" 🔌 Active Listening Ports ({}) ", filtered.len());
    let table = Table::new(rows, widths).header(header).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(THEME.border_active))
            .title(Span::styled(
                table_title,
                Style::default()
                    .fg(THEME.primary)
                    .add_modifier(Modifier::BOLD),
            )),
    );

    // Stateful rendering ensures the table scrolls as the cursor moves!
    frame.render_stateful_widget(table, chunks[0], &mut app.ports_table_state);

    // Render smooth vertical scrollbar
    if !filtered.is_empty() {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("▲"))
            .end_symbol(Some("▼"))
            .track_symbol(Some("│"))
            .thumb_symbol("█")
            .style(Style::default().fg(THEME.primary));
        let mut scrollbar_state =
            ScrollbarState::new(filtered.len().saturating_sub(1)).position(app.ports_selected);
        frame.render_stateful_widget(scrollbar, chunks[0], &mut scrollbar_state);
    }

    // Inspector pane for the selected port
    let inspect_content = if let Some(selected) = filtered.get(app.ports_selected) {
        vec![
            Line::from(vec![
                Span::styled("Selected: ", Style::default().fg(THEME.muted)),
                Span::styled(
                    format!(":{} ({})", selected.port, selected.protocol),
                    Style::default()
                        .fg(THEME.warning)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("  PID: ", Style::default().fg(THEME.muted)),
                Span::styled(
                    selected
                        .pid
                        .map(|n| n.to_string())
                        .unwrap_or_else(|| "N/A".to_string()),
                    Style::default().fg(THEME.secondary),
                ),
                Span::styled("  Process: ", Style::default().fg(THEME.muted)),
                Span::styled(
                    &selected.process_name,
                    Style::default().fg(THEME.text).add_modifier(Modifier::BOLD),
                ),
                Span::styled("  URL: ", Style::default().fg(THEME.muted)),
                Span::styled(selected.url(), Style::default().fg(THEME.accent)),
            ]),
            Line::from(vec![
                Span::styled("Cmd: ", Style::default().fg(THEME.muted)),
                Span::styled(
                    if selected.cmdline.is_empty() {
                        "(system / hidden)"
                    } else {
                        &selected.cmdline
                    },
                    Style::default().fg(THEME.text_dim),
                ),
            ]),
            Line::from(vec![
                Span::styled("Actions: ", Style::default().fg(THEME.muted)),
                Span::styled(
                    "[o] ",
                    Style::default()
                        .fg(THEME.primary)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("Open in Browser   ", Style::default().fg(THEME.text)),
                Span::styled(
                    "[x] ",
                    Style::default()
                        .fg(THEME.danger)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("Kill Port Process   ", Style::default().fg(THEME.text)),
                Span::styled(
                    "[Enter] ",
                    Style::default()
                        .fg(THEME.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("Full Process Details   ", Style::default().fg(THEME.text)),
                Span::styled(
                    "[r] ",
                    Style::default()
                        .fg(THEME.warning)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("Refresh", Style::default().fg(THEME.text)),
            ]),
        ]
    } else {
        vec![Line::from(Span::styled(
            "No ports matching filter.",
            Style::default().fg(THEME.muted),
        ))]
    };

    let inspect_box = Paragraph::new(inspect_content).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(THEME.border))
            .title(Span::styled(
                " 🔍 Quick Inspector & Actions ",
                Style::default()
                    .fg(THEME.secondary)
                    .add_modifier(Modifier::BOLD),
            )),
    );
    frame.render_widget(inspect_box, chunks[1]);
}
