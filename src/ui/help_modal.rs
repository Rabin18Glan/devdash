use ratatui::{
    Frame,
    layout::Alignment,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
};

use crate::app::App;
use crate::ui::logs_modal::centered_rect;
use crate::ui::theme::THEME;

pub fn render_help(frame: &mut Frame, app: &App) {
    if !app.help_modal_open {
        return;
    }

    let area = centered_rect(75, 85, frame.area());
    frame.render_widget(Clear, area);

    let shortcuts = vec![
        Line::from(Span::styled(
            "✨ Keyboard Shortcuts & Navigation",
            Style::default()
                .fg(THEME.primary)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  Tab / 1, 2, 3, 4, 5 ",
                Style::default()
                    .fg(THEME.warning)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Switch tab (Ports / Docker / Procs / Services / Storage)",
                Style::default().fg(THEME.text),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "  ↑ / ↓, j / k        ",
                Style::default()
                    .fg(THEME.warning)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Navigate rows (smooth scrolling with scrollbar)",
                Style::default().fg(THEME.text),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "  PgUp / PgDn         ",
                Style::default()
                    .fg(THEME.warning)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Scroll 10 rows at a time (or use Mouse Wheel)",
                Style::default().fg(THEME.text),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "  /                   ",
                Style::default()
                    .fg(THEME.warning)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Live search & filter (Esc to clear)",
                Style::default().fg(THEME.text),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "  s / S               ",
                Style::default()
                    .fg(THEME.warning)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Cycle sort order for active view",
                Style::default().fg(THEME.text),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "  r                   ",
                Style::default()
                    .fg(THEME.warning)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Force refresh all metrics, ports, services & disks",
                Style::default().fg(THEME.text),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "🔌 Ports Tab:",
            Style::default()
                .fg(THEME.accent)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled(
                "  o                   ",
                Style::default()
                    .fg(THEME.secondary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Open localhost URL in your browser",
                Style::default().fg(THEME.text),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "  x                   ",
                Style::default()
                    .fg(THEME.secondary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Kill process occupying the port",
                Style::default().fg(THEME.text),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "  p / n / m           ",
                Style::default()
                    .fg(THEME.secondary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Sort by Port / Name / Memory",
                Style::default().fg(THEME.text),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "⚙ Services Tab (systemctl background processes):",
            Style::default()
                .fg(THEME.accent)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled(
                "  d                   ",
                Style::default()
                    .fg(THEME.danger)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Disable --now (e.g. systemctl --user disable --now <service>)",
                Style::default().fg(THEME.text),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "  e                   ",
                Style::default()
                    .fg(THEME.success)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Enable --now (systemctl --user enable --now <service>)",
                Style::default().fg(THEME.text),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "  Space / s           ",
                Style::default()
                    .fg(THEME.secondary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Toggle Start / Stop", Style::default().fg(THEME.text)),
        ]),
        Line::from(vec![
            Span::styled(
                "  R                   ",
                Style::default()
                    .fg(THEME.secondary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Restart service", Style::default().fg(THEME.text)),
        ]),
        Line::from(vec![
            Span::styled(
                "  l                   ",
                Style::default()
                    .fg(THEME.secondary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "View Journal logs (journalctl -u <service>)",
                Style::default().fg(THEME.text),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "  u                   ",
                Style::default()
                    .fg(THEME.secondary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Toggle User services (--user) vs System services",
                Style::default().fg(THEME.text),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "💾 Storage Tab:",
            Style::default()
                .fg(THEME.accent)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled(
                "  u / t / m           ",
                Style::default()
                    .fg(THEME.secondary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Sort by Usage % / Total Size / Mount Point",
                Style::default().fg(THEME.text),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  ?                   ",
                Style::default()
                    .fg(THEME.warning)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Toggle this Help cheatsheet",
                Style::default().fg(THEME.text),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "  q / Ctrl+C          ",
                Style::default()
                    .fg(THEME.warning)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Quit DevDash", Style::default().fg(THEME.text)),
        ]),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(THEME.primary))
        .title(Span::styled(
            " 📖 Help & Keybindings ",
            Style::default()
                .fg(THEME.accent)
                .add_modifier(Modifier::BOLD),
        ));

    let p = Paragraph::new(shortcuts).block(block);
    frame.render_widget(p, area);
}

pub fn render_confirm_kill(frame: &mut Frame, app: &App) {
    if let Some(target) = &app.confirm_kill_target {
        let area = centered_rect(50, 30, frame.area());
        frame.render_widget(Clear, area);

        let lines = vec![
            Line::from(""),
            Line::from(Span::styled(
                "⚠️  Confirm Process Termination",
                Style::default()
                    .fg(THEME.warning)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled("Target:  ", Style::default().fg(THEME.muted)),
                Span::styled(
                    &target.name,
                    Style::default().fg(THEME.text).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(" (PID {})", target.pid),
                    Style::default().fg(THEME.secondary),
                ),
            ]),
            if let Some(port) = &target.port_info {
                Line::from(vec![
                    Span::styled("Port:    ", Style::default().fg(THEME.muted)),
                    Span::styled(port, Style::default().fg(THEME.warning)),
                ])
            } else {
                Line::from("")
            },
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    " [y / Enter] ",
                    Style::default()
                        .bg(THEME.danger)
                        .fg(Color::Black)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" Terminate (SIGTERM)  ", Style::default().fg(THEME.text)),
                Span::styled(
                    " [F] ",
                    Style::default()
                        .bg(Color::Red)
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" Force Kill  ", Style::default().fg(THEME.text)),
                Span::styled(
                    " [Esc / n] ",
                    Style::default().bg(THEME.muted).fg(Color::White),
                ),
                Span::styled(" Cancel", Style::default().fg(THEME.text)),
            ]),
        ];

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(THEME.danger))
            .title(Span::styled(
                " 🛑 Kill Process ",
                Style::default()
                    .fg(THEME.danger)
                    .add_modifier(Modifier::BOLD),
            ));

        let p = Paragraph::new(lines)
            .block(block)
            .alignment(Alignment::Center);
        frame.render_widget(p, area);
    }
}
