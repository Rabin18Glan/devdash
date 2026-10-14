use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
};

use crate::app::{ActiveTab, App};
use crate::ui::theme::THEME;

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(18), // Logo
            Constraint::Min(55),    // 5 Tabs
            Constraint::Length(28), // Clock / Search
        ])
        .split(area);

    // 1. Logo
    let logo = Line::from(vec![
        Span::styled("⚡ ", Style::default().fg(THEME.warning)),
        Span::styled(
            "DEV",
            Style::default()
                .fg(THEME.primary)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "DASH",
            Style::default()
                .fg(THEME.accent)
                .add_modifier(Modifier::BOLD),
        ),
    ]);
    let logo_widget = Paragraph::new(logo).block(Block::default().borders(Borders::NONE));
    frame.render_widget(logo_widget, chunks[0]);

    // 2. 5 Tabs
    let tab_titles = vec![
        format!(" [1] Ports ({}) ", app.filtered_ports().len()),
        format!(
            " [2] Docker ({}) ",
            if app.docker_available {
                app.filtered_docker().len().to_string()
            } else {
                "off".to_string()
            }
        ),
        format!(" [3] Procs ({}) ", app.filtered_processes().len()),
        format!(" [4] Services ({}) ", app.filtered_services().len()),
        format!(" [5] Storage ({}) ", app.filtered_storage().len()),
    ];

    let selected_index = match app.active_tab {
        ActiveTab::Ports => 0,
        ActiveTab::Docker => 1,
        ActiveTab::Processes => 2,
        ActiveTab::Services => 3,
        ActiveTab::Storage => 4,
    };

    let tabs_widget = Tabs::new(tab_titles)
        .select(selected_index)
        .style(Style::default().fg(THEME.muted))
        .highlight_style(
            Style::default()
                .fg(THEME.primary)
                .bg(THEME.highlight_bg)
                .add_modifier(Modifier::BOLD),
        )
        .divider(Span::styled("│", Style::default().fg(THEME.border)));
    frame.render_widget(tabs_widget, chunks[1]);

    // 3. Search / Clock
    let right_line = if app.is_searching {
        Line::from(vec![
            Span::styled("🔍 ", Style::default().fg(THEME.warning)),
            Span::styled(
                &app.search_query,
                Style::default()
                    .fg(THEME.text)
                    .add_modifier(Modifier::UNDERLINED),
            ),
            Span::styled("█", Style::default().fg(THEME.primary)),
        ])
    } else if !app.search_query.is_empty() {
        Line::from(vec![
            Span::styled("🔍 Filter: ", Style::default().fg(THEME.muted)),
            Span::styled(&app.search_query, Style::default().fg(THEME.accent)),
        ])
    } else {
        let now = chrono::Local::now().format("%H:%M:%S").to_string();
        Line::from(vec![
            Span::styled("● ", Style::default().fg(THEME.success)),
            Span::styled(&app.metrics.host_name, Style::default().fg(THEME.text_dim)),
            Span::styled(" │ ", Style::default().fg(THEME.border)),
            Span::styled(now, Style::default().fg(THEME.secondary)),
        ])
    };

    let right_widget = Paragraph::new(right_line).alignment(Alignment::Right);
    frame.render_widget(right_widget, chunks[2]);
}
