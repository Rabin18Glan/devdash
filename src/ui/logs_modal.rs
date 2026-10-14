use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
};

use crate::app::App;
use crate::ui::theme::THEME;

pub fn render(frame: &mut Frame, app: &App) {
    if !app.logs_modal_open {
        return;
    }

    let area = centered_rect(85, 80, frame.area());
    frame.render_widget(Clear, area); // Clears the background behind the modal

    let total_lines = app.logs_content.len();
    let visible_height = area.height.saturating_sub(4) as usize;

    let title = format!(" {} ({} lines) ", app.logs_title, total_lines);
    let scroll_pos = app
        .logs_scroll
        .min(total_lines.saturating_sub(visible_height));

    let visible_lines: Vec<Line> = app
        .logs_content
        .iter()
        .skip(scroll_pos)
        .take(visible_height)
        .map(|l| {
            if l.contains("ERROR") || l.contains("err") || l.contains("Fatal") {
                Line::from(Span::styled(l, Style::default().fg(THEME.danger)))
            } else if l.contains("WARN") {
                Line::from(Span::styled(l, Style::default().fg(THEME.warning)))
            } else if l.contains("INFO") {
                Line::from(Span::styled(l, Style::default().fg(THEME.text)))
            } else {
                Line::from(Span::styled(l, Style::default().fg(THEME.text_dim)))
            }
        })
        .collect();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Length(1)])
        .split(area);

    let modal_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(THEME.primary))
        .title(Span::styled(
            title,
            Style::default()
                .fg(THEME.accent)
                .add_modifier(Modifier::BOLD),
        ));

    let p = Paragraph::new(visible_lines).block(modal_block);
    frame.render_widget(p, chunks[0]);

    // Scroll helper footer
    let footer_text = Line::from(vec![
        Span::styled(
            " [↑/↓, j/k, PgUp/PgDn] ",
            Style::default()
                .fg(THEME.primary)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Scroll   ", Style::default().fg(THEME.text)),
        Span::styled(
            " [g / G] ",
            Style::default()
                .fg(THEME.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Top / Bottom   ", Style::default().fg(THEME.text)),
        Span::styled(
            " [Esc / q / Enter] ",
            Style::default()
                .fg(THEME.warning)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Close", Style::default().fg(THEME.text)),
    ]);
    let footer = Paragraph::new(footer_text)
        .alignment(Alignment::Center)
        .style(Style::default().bg(Color::Rgb(20, 22, 34)));
    frame.render_widget(footer, chunks[1]);
}

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
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
