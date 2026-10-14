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

use crate::app::{App, StorageSort};
use crate::ui::theme::THEME;

pub fn render(frame: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(8),    // Disks Table
            Constraint::Length(5), // Overview card
        ])
        .split(area);

    let filtered = app.filtered_storage();

    // Table Header
    let header_cells = [
        "",
        "Mount Point",
        "Device Name",
        "Filesystem",
        "Used / Total",
        "Usage %",
        "Free Space",
        "Removable",
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
        .map(|(i, d)| {
            let is_selected = i == app.storage_selected;
            let arrow = if is_selected { "▶" } else { " " };

            let pct = d.usage_percent.clamp(0.0, 100.0);
            let bar_color = if pct < 70.0 {
                THEME.success
            } else if pct < 85.0 {
                THEME.warning
            } else {
                THEME.danger
            };

            // Build a visual bar: e.g. [██████░░░░] 60%
            let filled = (pct / 10.0).round() as usize;
            let empty = 10usize.saturating_sub(filled);
            let bar_str = format!("{}{} {:.0}%", "█".repeat(filled), "░".repeat(empty), pct);

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
                    &d.mount_point,
                    Style::default()
                        .fg(THEME.accent)
                        .add_modifier(Modifier::BOLD),
                )),
                Cell::from(Span::styled(&d.name, Style::default().fg(THEME.text_dim))),
                Cell::from(Span::styled(
                    &d.file_system,
                    Style::default().fg(THEME.secondary),
                )),
                Cell::from(Span::styled(
                    format!("{:.1} G / {:.1} G", d.used_gb, d.total_gb),
                    Style::default().fg(THEME.text),
                )),
                Cell::from(Span::styled(
                    bar_str,
                    Style::default().fg(bar_color).add_modifier(Modifier::BOLD),
                )),
                Cell::from(Span::styled(
                    format!("{:.1} GB", d.free_gb),
                    Style::default().fg(THEME.primary),
                )),
                Cell::from(Span::styled(
                    if d.is_removable { "Yes (USB)" } else { "No" },
                    Style::default().fg(THEME.text_dim),
                )),
            ])
            .style(row_style)
            .height(1)
        })
        .collect();

    let widths = [
        Constraint::Length(2),  // Arrow
        Constraint::Length(20), // Mount
        Constraint::Length(26), // Device
        Constraint::Length(12), // FS
        Constraint::Length(18), // Used / Total
        Constraint::Length(18), // Usage bar
        Constraint::Length(14), // Free
        Constraint::Length(10), // Removable
    ];

    let sort_str = match app.storage_sort {
        StorageSort::Total => "Sorted by Total Size",
        StorageSort::Usage => "Sorted by Usage %",
        StorageSort::Mount => "Sorted by Mount",
    };

    let table_title = format!(
        " 💾 Storage Disks & Mounts ({}) │ [{}] ",
        filtered.len(),
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

    frame.render_stateful_widget(table, chunks[0], &mut app.storage_table_state);

    if !filtered.is_empty() {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("▲"))
            .end_symbol(Some("▼"))
            .track_symbol(Some("│"))
            .thumb_symbol("█")
            .style(Style::default().fg(THEME.primary));
        let mut scrollbar_state =
            ScrollbarState::new(filtered.len().saturating_sub(1)).position(app.storage_selected);
        frame.render_stateful_widget(scrollbar, chunks[0], &mut scrollbar_state);
    }

    // Storage summary totals
    let total_all_gb: f64 = app.storage_items.iter().map(|d| d.total_gb).sum();
    let used_all_gb: f64 = app.storage_items.iter().map(|d| d.used_gb).sum();
    let free_all_gb: f64 = app.storage_items.iter().map(|d| d.free_gb).sum();
    let overall_pct = if total_all_gb > 0.0 {
        (used_all_gb / total_all_gb) * 100.0
    } else {
        0.0
    };

    let summary_lines = vec![
        Line::from(vec![
            Span::styled("Total Capacity: ", Style::default().fg(THEME.muted)),
            Span::styled(
                format!("{:.1} GB", total_all_gb),
                Style::default()
                    .fg(THEME.primary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("   Used: ", Style::default().fg(THEME.muted)),
            Span::styled(
                format!("{:.1} GB ({:.1}%)", used_all_gb, overall_pct),
                Style::default()
                    .fg(THEME.warning)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("   Available Free: ", Style::default().fg(THEME.muted)),
            Span::styled(
                format!("{:.1} GB", free_all_gb),
                Style::default()
                    .fg(THEME.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("   Mounts: ", Style::default().fg(THEME.muted)),
            Span::styled(
                app.storage_items.len().to_string(),
                Style::default().fg(THEME.secondary),
            ),
        ]),
        Line::from(vec![
            Span::styled("Sorting: ", Style::default().fg(THEME.muted)),
            Span::styled(
                "[u] ",
                Style::default()
                    .fg(THEME.primary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Sort by Usage %   ", Style::default().fg(THEME.text)),
            Span::styled(
                "[t] ",
                Style::default()
                    .fg(THEME.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Sort by Total Size   ", Style::default().fg(THEME.text)),
            Span::styled(
                "[m] ",
                Style::default()
                    .fg(THEME.secondary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Sort by Mount Point   ", Style::default().fg(THEME.text)),
            Span::styled(
                "[r] ",
                Style::default()
                    .fg(THEME.warning)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Refresh Disks", Style::default().fg(THEME.text)),
        ]),
    ];

    let summary_box = Paragraph::new(summary_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(THEME.border))
            .title(Span::styled(
                " 💾 Storage Capacity Summary ",
                Style::default()
                    .fg(THEME.secondary)
                    .add_modifier(Modifier::BOLD),
            )),
    );
    frame.render_widget(summary_box, chunks[1]);
}
