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

use crate::app::{App, ProcessSort};
use crate::ui::theme::THEME;

pub fn render(frame: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(8),    // Process Table
            Constraint::Length(5), // Inspector / Sort toolbar
        ])
        .split(area);

    let filtered = app.filtered_processes();

    // Table Header
    let header_cells = [
        "",
        "PID",
        "Process Name",
        "CPU%",
        "Memory",
        "Type",
        "Command Line",
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
            let is_selected = i == app.proc_selected;
            let arrow = if is_selected { "▶" } else { " " };

            let type_span = if p.is_dev_related {
                Span::styled(
                    "⚡ Dev",
                    Style::default()
                        .fg(THEME.accent)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Span::styled("System", Style::default().fg(THEME.text_dim))
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
                Cell::from(Span::styled(
                    p.pid.to_string(),
                    Style::default().fg(THEME.secondary),
                )),
                Cell::from(Span::styled(
                    &p.name,
                    Style::default().fg(THEME.text).add_modifier(Modifier::BOLD),
                )),
                Cell::from(Span::styled(
                    format!("{:.1}%", p.cpu_usage),
                    Style::default().fg(THEME.warning),
                )),
                Cell::from(Span::styled(
                    format!("{:.1} MB", p.memory_mb),
                    Style::default().fg(THEME.accent),
                )),
                Cell::from(type_span),
                Cell::from(Span::styled(
                    &p.cmdline,
                    Style::default().fg(THEME.text_dim),
                )),
            ])
            .style(row_style)
            .height(1)
        })
        .collect();

    let widths = [
        Constraint::Length(2),  // Arrow
        Constraint::Length(9),  // PID
        Constraint::Length(22), // Name
        Constraint::Length(9),  // CPU%
        Constraint::Length(12), // Mem
        Constraint::Length(10), // Type
        Constraint::Min(30),    // Cmdline
    ];

    let filter_badge = if app.dev_only_processes {
        "Dev Only"
    } else {
        "All"
    };
    let sort_badge = match app.process_sort {
        ProcessSort::Cpu => "Sorted by CPU",
        ProcessSort::Memory => "Sorted by Memory",
        ProcessSort::Pid => "Sorted by PID",
        ProcessSort::Name => "Sorted by Name",
    };

    let table_title = format!(
        " ⚙ Active Processes ({} shown) │ [{}] │ [{}] ",
        filtered.len(),
        filter_badge,
        sort_badge
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

    // Stateful rendering for smooth scrolling
    frame.render_stateful_widget(table, chunks[0], &mut app.proc_table_state);

    if !filtered.is_empty() {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("▲"))
            .end_symbol(Some("▼"))
            .track_symbol(Some("│"))
            .thumb_symbol("█")
            .style(Style::default().fg(THEME.primary));
        let mut scrollbar_state =
            ScrollbarState::new(filtered.len().saturating_sub(1)).position(app.proc_selected);
        frame.render_stateful_widget(scrollbar, chunks[0], &mut scrollbar_state);
    }

    // Bottom action / details pane
    let inspect_content = if let Some(selected) = filtered.get(app.proc_selected) {
        vec![
            Line::from(vec![
                Span::styled("Selected: ", Style::default().fg(THEME.muted)),
                Span::styled(
                    &selected.name,
                    Style::default().fg(THEME.text).add_modifier(Modifier::BOLD),
                ),
                Span::styled(" (PID ", Style::default().fg(THEME.muted)),
                Span::styled(
                    selected.pid.to_string(),
                    Style::default().fg(THEME.secondary),
                ),
                Span::styled(")  CPU: ", Style::default().fg(THEME.muted)),
                Span::styled(
                    format!("{:.1}%", selected.cpu_usage),
                    Style::default().fg(THEME.warning),
                ),
                Span::styled("  Memory: ", Style::default().fg(THEME.muted)),
                Span::styled(
                    format!("{:.1} MB", selected.memory_mb),
                    Style::default().fg(THEME.accent),
                ),
            ]),
            Line::from(vec![
                Span::styled("Cmd: ", Style::default().fg(THEME.muted)),
                Span::styled(
                    if selected.cmdline.is_empty() {
                        "(no arguments)"
                    } else {
                        &selected.cmdline
                    },
                    Style::default().fg(THEME.text_dim),
                ),
            ]),
            Line::from(vec![
                Span::styled("Controls: ", Style::default().fg(THEME.muted)),
                Span::styled(
                    "[x] ",
                    Style::default()
                        .fg(THEME.danger)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("Kill Process   ", Style::default().fg(THEME.text)),
                Span::styled(
                    "[a] ",
                    Style::default()
                        .fg(THEME.primary)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("Toggle Dev/All   ", Style::default().fg(THEME.text)),
                Span::styled(
                    "[c] ",
                    Style::default()
                        .fg(THEME.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("Sort CPU   ", Style::default().fg(THEME.text)),
                Span::styled(
                    "[m] ",
                    Style::default()
                        .fg(THEME.secondary)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("Sort Mem   ", Style::default().fg(THEME.text)),
                Span::styled(
                    "[Enter] ",
                    Style::default()
                        .fg(THEME.warning)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("Inspect Details", Style::default().fg(THEME.text)),
            ]),
        ]
    } else {
        vec![Line::from(Span::styled(
            "No processes match filter.",
            Style::default().fg(THEME.muted),
        ))]
    };

    let inspect_box = Paragraph::new(inspect_content).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(THEME.border))
            .title(Span::styled(
                " 🔍 Process Controls ",
                Style::default()
                    .fg(THEME.secondary)
                    .add_modifier(Modifier::BOLD),
            )),
    );
    frame.render_widget(inspect_box, chunks[1]);
}
