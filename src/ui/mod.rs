pub mod docker_view;
pub mod header;
pub mod help_modal;
pub mod hud;
pub mod logs_modal;
pub mod ports_view;
pub mod process_view;
pub mod services_view;
pub mod storage_view;
pub mod theme;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use std::time::Duration;

use self::theme::THEME;
use crate::app::{ActiveTab, App};

pub fn render(frame: &mut Frame, app: &mut App) {
    let size = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Header bar & tabs
            Constraint::Length(4), // System HUD (gauges & load)
            Constraint::Min(10),   // Active View content
            Constraint::Length(1), // Footer status bar
        ])
        .split(size);

    // 1. Header
    header::render(frame, app, chunks[0]);

    // 2. HUD
    hud::render(frame, app, chunks[1]);

    // 3. Tab Content
    match app.active_tab {
        ActiveTab::Ports => ports_view::render(frame, app, chunks[2]),
        ActiveTab::Docker => docker_view::render(frame, app, chunks[2]),
        ActiveTab::Processes => process_view::render(frame, app, chunks[2]),
        ActiveTab::Services => services_view::render(frame, app, chunks[2]),
        ActiveTab::Storage => storage_view::render(frame, app, chunks[2]),
    }

    // 4. Footer & Notification Toast
    render_footer(frame, app, chunks[3]);

    // 5. Overlays & Modals
    logs_modal::render(frame, app);
    help_modal::render_confirm_kill(frame, app);
    help_modal::render_help(frame, app);
}

fn render_footer(frame: &mut Frame, app: &App, area: Rect) {
    let now = std::time::Instant::now();
    let toast = app.status_message.as_ref().and_then(|(msg, time)| {
        if now.duration_since(*time) < Duration::from_secs(5) {
            Some(msg.as_str())
        } else {
            None
        }
    });

    let footer_line = if let Some(msg) = toast {
        let is_err = msg.starts_with('✖');
        let color = if is_err { THEME.danger } else { THEME.accent };
        Line::from(vec![
            Span::styled(" ", Style::default()),
            Span::styled(msg, Style::default().fg(color).add_modifier(Modifier::BOLD)),
        ])
    } else {
        Line::from(vec![
            Span::styled(
                " [Tab/1-5] ",
                Style::default()
                    .fg(THEME.primary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Tabs ", Style::default().fg(THEME.text_dim)),
            Span::styled(
                " [↑↓/jk] ",
                Style::default()
                    .fg(THEME.primary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Move ", Style::default().fg(THEME.text_dim)),
            Span::styled(
                " [/] ",
                Style::default()
                    .fg(THEME.primary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Search ", Style::default().fg(THEME.text_dim)),
            Span::styled(
                " [s] ",
                Style::default()
                    .fg(THEME.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Cycle Sort ", Style::default().fg(THEME.text_dim)),
            Span::styled(
                " [x] ",
                Style::default()
                    .fg(THEME.danger)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Kill ", Style::default().fg(THEME.text_dim)),
            Span::styled(
                " [d] ",
                Style::default()
                    .fg(THEME.warning)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Disable--now ", Style::default().fg(THEME.text_dim)),
            Span::styled(
                " [r] ",
                Style::default()
                    .fg(THEME.warning)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Refresh ", Style::default().fg(THEME.text_dim)),
            Span::styled(
                " [?] ",
                Style::default()
                    .fg(THEME.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Help ", Style::default().fg(THEME.text_dim)),
            Span::styled(
                " [q] ",
                Style::default()
                    .fg(THEME.secondary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Quit ", Style::default().fg(THEME.text_dim)),
        ])
    };

    let p = Paragraph::new(footer_line)
        .style(Style::default().bg(Color::Rgb(18, 20, 30)))
        .block(Block::default().borders(Borders::NONE));
    frame.render_widget(p, area);
}
