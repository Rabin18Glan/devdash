use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Gauge, Paragraph},
};

use crate::app::App;
use crate::ui::theme::THEME;

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(28), // CPU Gauge
            Constraint::Percentage(28), // Memory Gauge
            Constraint::Percentage(22), // Load & OS info
            Constraint::Percentage(22), // Dev Summary
        ])
        .split(area);

    // 1. CPU Gauge
    let cpu_pct = app.metrics.cpu_usage.clamp(0.0, 100.0);
    let cpu_color = if cpu_pct < 50.0 {
        THEME.success
    } else if cpu_pct < 80.0 {
        THEME.warning
    } else {
        THEME.danger
    };

    let cpu_gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(THEME.border))
                .title(Span::styled(
                    " ⚙ CPU Usage ",
                    Style::default()
                        .fg(THEME.primary)
                        .add_modifier(Modifier::BOLD),
                )),
        )
        .gauge_style(Style::default().fg(cpu_color).bg(Color::Rgb(25, 27, 40)))
        .percent(cpu_pct as u16)
        .label(format!("{:.1}%", cpu_pct));
    frame.render_widget(cpu_gauge, chunks[0]);

    // 2. Memory Gauge
    let mem_pct = app.metrics.memory_percent.clamp(0.0, 100.0);
    let mem_color = if mem_pct < 60.0 {
        THEME.accent
    } else if mem_pct < 85.0 {
        THEME.warning
    } else {
        THEME.danger
    };

    let mem_label = format!(
        "{:.1}G / {:.1}G ({:.0}%)",
        app.metrics.used_memory_mb as f64 / 1024.0,
        app.metrics.total_memory_mb as f64 / 1024.0,
        mem_pct
    );

    let mem_gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(THEME.border))
                .title(Span::styled(
                    " 💾 Memory ",
                    Style::default()
                        .fg(THEME.secondary)
                        .add_modifier(Modifier::BOLD),
                )),
        )
        .gauge_style(Style::default().fg(mem_color).bg(Color::Rgb(25, 27, 40)))
        .percent(mem_pct as u16)
        .label(mem_label);
    frame.render_widget(mem_gauge, chunks[1]);

    // 3. Load & OS Info
    let load_text = vec![
        Line::from(vec![
            Span::styled("Load: ", Style::default().fg(THEME.muted)),
            Span::styled(
                format!(
                    "{:.2}  {:.2}  {:.2}",
                    app.metrics.load_avg_one,
                    app.metrics.load_avg_five,
                    app.metrics.load_avg_fifteen
                ),
                Style::default().fg(THEME.text),
            ),
        ]),
        Line::from(vec![
            Span::styled("Up:   ", Style::default().fg(THEME.muted)),
            Span::styled(
                format_uptime(app.metrics.uptime_secs),
                Style::default().fg(THEME.text_dim),
            ),
        ]),
    ];
    let load_widget = Paragraph::new(load_text).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(THEME.border))
            .title(Span::styled(
                " 📊 System Load ",
                Style::default()
                    .fg(THEME.accent)
                    .add_modifier(Modifier::BOLD),
            )),
    );
    frame.render_widget(load_widget, chunks[2]);

    // 4. Quick Summary
    let running_containers = app
        .docker_containers
        .iter()
        .filter(|c| c.is_running())
        .count();
    let total_containers = app.docker_containers.len();
    let docker_status_span = if app.docker_available {
        Span::styled(
            format!("{}/{} running", running_containers, total_containers),
            if running_containers > 0 {
                Style::default().fg(THEME.success)
            } else {
                Style::default().fg(THEME.text_dim)
            },
        )
    } else {
        Span::styled("Offline / Disabled", Style::default().fg(THEME.danger))
    };

    let summary_text = vec![
        Line::from(vec![
            Span::styled("Ports:  ", Style::default().fg(THEME.muted)),
            Span::styled(
                format!("{} listening", app.ports.len()),
                Style::default().fg(THEME.primary),
            ),
        ]),
        Line::from(vec![
            Span::styled("Docker: ", Style::default().fg(THEME.muted)),
            docker_status_span,
        ]),
    ];
    let summary_widget = Paragraph::new(summary_text).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(THEME.border))
            .title(Span::styled(
                " 🚀 Overview ",
                Style::default()
                    .fg(THEME.primary)
                    .add_modifier(Modifier::BOLD),
            )),
    );
    frame.render_widget(summary_widget, chunks[3]);
}

fn format_uptime(secs: u64) -> String {
    let days = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let mins = (secs % 3600) / 60;
    if days > 0 {
        format!("{}d {}h {}m", days, hours, mins)
    } else if hours > 0 {
        format!("{}h {}m", hours, mins)
    } else {
        format!("{}m", mins)
    }
}
