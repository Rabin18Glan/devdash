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

use crate::app::{App, ServicesSort};
use crate::ui::theme::THEME;

pub fn render(frame: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(8),    // Services Table
            Constraint::Length(5), // Action / Controls toolbar
        ])
        .split(area);

    let filtered = app.filtered_services();

    // Table Header
    let header_cells = ["", "State", "Unit Name", "Substate", "Description"]
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
        .map(|(i, s)| {
            let is_selected = i == app.services_selected;
            let arrow = if is_selected { "▶" } else { " " };

            let (state_symbol, state_style) = if s.is_running() {
                (
                    "● RUNNING",
                    Style::default()
                        .fg(THEME.success)
                        .add_modifier(Modifier::BOLD),
                )
            } else if s.is_failed() {
                (
                    "✖ FAILED",
                    Style::default()
                        .fg(THEME.danger)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                ("⏹ DEAD", Style::default().fg(THEME.muted))
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
                    &s.unit,
                    Style::default().fg(THEME.text).add_modifier(Modifier::BOLD),
                )),
                Cell::from(Span::styled(&s.sub, Style::default().fg(THEME.secondary))),
                Cell::from(Span::styled(
                    &s.description,
                    Style::default().fg(THEME.text_dim),
                )),
            ])
            .style(row_style)
            .height(1)
        })
        .collect();

    let widths = [
        Constraint::Length(2),  // Arrow
        Constraint::Length(12), // State
        Constraint::Length(34), // Unit Name
        Constraint::Length(12), // Substate
        Constraint::Min(30),    // Description
    ];

    let mode_str = if app.services_user_mode {
        "systemctl --user"
    } else {
        "systemctl (system)"
    };
    let sort_str = match app.services_sort {
        ServicesSort::Status => "Sorted by Status",
        ServicesSort::Name => "Sorted by Name",
    };

    let table_title = format!(
        " ⚙ Background Services ({}) │ [{}] │ [{}] ",
        filtered.len(),
        mode_str,
        sort_str
    );
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

    frame.render_stateful_widget(table, chunks[0], &mut app.services_table_state);

    if !filtered.is_empty() {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("▲"))
            .end_symbol(Some("▼"))
            .track_symbol(Some("│"))
            .thumb_symbol("█")
            .style(Style::default().fg(THEME.primary));
        let mut scrollbar_state =
            ScrollbarState::new(filtered.len().saturating_sub(1)).position(app.services_selected);
        frame.render_stateful_widget(scrollbar, chunks[0], &mut scrollbar_state);
    }

    // Inspector & Actions
    let inspect_content = if let Some(selected) = filtered.get(app.services_selected) {
        vec![
            Line::from(vec![
                Span::styled("Selected: ", Style::default().fg(THEME.muted)),
                Span::styled(
                    &selected.unit,
                    Style::default()
                        .fg(THEME.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("  Status: ", Style::default().fg(THEME.muted)),
                Span::styled(
                    format!("{} ({})", selected.active, selected.sub),
                    if selected.is_running() {
                        Style::default().fg(THEME.success)
                    } else {
                        Style::default().fg(THEME.danger)
                    },
                ),
                Span::styled("  Scope: ", Style::default().fg(THEME.muted)),
                Span::styled(
                    if app.services_user_mode {
                        "User (--user)"
                    } else {
                        "System"
                    },
                    Style::default().fg(THEME.secondary),
                ),
            ]),
            Line::from(vec![
                Span::styled("Desc:     ", Style::default().fg(THEME.muted)),
                Span::styled(&selected.description, Style::default().fg(THEME.text_dim)),
            ]),
            Line::from(vec![
                Span::styled("Controls: ", Style::default().fg(THEME.muted)),
                Span::styled(
                    "[d] ",
                    Style::default()
                        .fg(THEME.danger)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("Disable --now   ", Style::default().fg(THEME.text)),
                Span::styled(
                    "[e] ",
                    Style::default()
                        .fg(THEME.success)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("Enable --now   ", Style::default().fg(THEME.text)),
                Span::styled(
                    "[s] ",
                    Style::default()
                        .fg(THEME.primary)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    if selected.is_running() {
                        "Stop   "
                    } else {
                        "Start   "
                    },
                    Style::default().fg(THEME.text),
                ),
                Span::styled(
                    "[R] ",
                    Style::default()
                        .fg(THEME.warning)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("Restart   ", Style::default().fg(THEME.text)),
                Span::styled(
                    "[l] ",
                    Style::default()
                        .fg(THEME.secondary)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("Journal Logs   ", Style::default().fg(THEME.text)),
                Span::styled(
                    "[u] ",
                    Style::default()
                        .fg(THEME.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("Toggle User/System", Style::default().fg(THEME.text)),
            ]),
        ]
    } else {
        vec![Line::from(Span::styled(
            "No services matching filter.",
            Style::default().fg(THEME.muted),
        ))]
    };

    let inspect_box = Paragraph::new(inspect_content).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(THEME.border))
            .title(Span::styled(
                " ⚙ Service Actions & Controls ",
                Style::default()
                    .fg(THEME.secondary)
                    .add_modifier(Modifier::BOLD),
            )),
    );
    frame.render_widget(inspect_box, chunks[1]);
}
