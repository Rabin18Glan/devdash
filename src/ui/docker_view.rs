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
    if !app.docker_available {
        let msg = vec![
            Line::from(""),
            Line::from(Span::styled(
                "🐳 Docker is not accessible or daemon is stopped.",
                Style::default()
                    .fg(THEME.warning)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                "Make sure Docker is running and your user has permissions (`sudo usermod -aG docker $USER`).",
                Style::default().fg(THEME.muted),
            )),
        ];
        let p = Paragraph::new(msg).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(THEME.border))
                .title(Span::styled(
                    " 🐳 Docker Containers ",
                    Style::default().fg(THEME.primary),
                )),
        );
        frame.render_widget(p, area);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(8),    // Containers Table
            Constraint::Length(5), // Action / Detail pane
        ])
        .split(area);

    let filtered = app.filtered_docker();

    // Table Header
    let header_cells = [
        "",
        "State",
        "Container Name",
        "Image",
        "Status",
        "Port Mappings",
        "ID",
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
        .map(|(i, c)| {
            let is_selected = i == app.docker_selected;
            let arrow = if is_selected { "▶" } else { " " };

            let (state_symbol, state_style) = if c.is_running() {
                (
                    "● RUNNING",
                    Style::default()
                        .fg(THEME.success)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                (
                    "⏹ STOPPED",
                    Style::default()
                        .fg(THEME.danger)
                        .add_modifier(Modifier::BOLD),
                )
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
                Cell::from(Span::styled(state_symbol, state_style)),
                Cell::from(Span::styled(
                    &c.names,
                    Style::default().fg(THEME.text).add_modifier(Modifier::BOLD),
                )),
                Cell::from(Span::styled(&c.image, Style::default().fg(THEME.secondary))),
                Cell::from(Span::styled(&c.status, Style::default().fg(THEME.text_dim))),
                Cell::from(Span::styled(&c.ports, Style::default().fg(THEME.warning))),
                Cell::from(Span::styled(
                    c.short_id(),
                    Style::default().fg(THEME.accent),
                )),
            ])
            .style(row_style)
            .height(1)
        })
        .collect();

    let widths = [
        Constraint::Length(2),  // Arrow
        Constraint::Length(12), // State
        Constraint::Length(24), // Name
        Constraint::Length(22), // Image
        Constraint::Length(18), // Status
        Constraint::Min(24),    // Ports
        Constraint::Length(14), // ID
    ];

    let table_title = format!(" 🐳 Docker Containers ({}) ", filtered.len());
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

    // Stateful rendering for smooth scrolling
    frame.render_stateful_widget(table, chunks[0], &mut app.docker_table_state);

    if !filtered.is_empty() {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("▲"))
            .end_symbol(Some("▼"))
            .track_symbol(Some("│"))
            .thumb_symbol("█")
            .style(Style::default().fg(THEME.primary));
        let mut scrollbar_state =
            ScrollbarState::new(filtered.len().saturating_sub(1)).position(app.docker_selected);
        frame.render_stateful_widget(scrollbar, chunks[0], &mut scrollbar_state);
    }

    // Inspector & Actions
    let inspect_content = if let Some(selected) = filtered.get(app.docker_selected) {
        vec![
            Line::from(vec![
                Span::styled("Selected: ", Style::default().fg(THEME.muted)),
                Span::styled(
                    &selected.names,
                    Style::default()
                        .fg(THEME.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("  Image: ", Style::default().fg(THEME.muted)),
                Span::styled(&selected.image, Style::default().fg(THEME.secondary)),
                Span::styled("  Ports: ", Style::default().fg(THEME.muted)),
                Span::styled(
                    if selected.ports.is_empty() {
                        "none"
                    } else {
                        &selected.ports
                    },
                    Style::default().fg(THEME.warning),
                ),
            ]),
            Line::from(vec![
                Span::styled("Command: ", Style::default().fg(THEME.muted)),
                Span::styled(&selected.command, Style::default().fg(THEME.text_dim)),
            ]),
            Line::from(vec![
                Span::styled("Actions: ", Style::default().fg(THEME.muted)),
                Span::styled(
                    "[s] ",
                    Style::default()
                        .fg(THEME.primary)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    if selected.is_running() {
                        "Stop Container   "
                    } else {
                        "Start Container   "
                    },
                    Style::default().fg(THEME.text),
                ),
                Span::styled(
                    "[R] ",
                    Style::default()
                        .fg(THEME.warning)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("Restart Container   ", Style::default().fg(THEME.text)),
                Span::styled(
                    "[l / Enter] ",
                    Style::default()
                        .fg(THEME.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("View Container Logs", Style::default().fg(THEME.text)),
            ]),
        ]
    } else {
        vec![Line::from(Span::styled(
            "No containers found.",
            Style::default().fg(THEME.muted),
        ))]
    };

    let inspect_box = Paragraph::new(inspect_content).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(THEME.border))
            .title(Span::styled(
                " ⚙ Container Controls ",
                Style::default()
                    .fg(THEME.secondary)
                    .add_modifier(Modifier::BOLD),
            )),
    );
    frame.render_widget(inspect_box, chunks[1]);
}
