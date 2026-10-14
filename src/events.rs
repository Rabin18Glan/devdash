use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};
use std::time::Duration;

use crate::app::{ActiveTab, App, DockerSort, PortsSort, ProcessSort, StorageSort};

pub fn handle_events(app: &mut App) -> anyhow::Result<()> {
    if event::poll(Duration::from_millis(100))? {
        match event::read()? {
            Event::Key(key) => handle_key_event(app, key),
            Event::Mouse(mouse) => handle_mouse_event(app, mouse),
            _ => {}
        }
    }
    Ok(())
}

fn handle_mouse_event(app: &mut App, mouse: MouseEvent) {
    if app.logs_modal_open {
        match mouse.kind {
            MouseEventKind::ScrollDown => {
                if app.logs_scroll + 1 < app.logs_content.len() {
                    app.logs_scroll += 1;
                }
            }
            MouseEventKind::ScrollUp => {
                if app.logs_scroll > 0 {
                    app.logs_scroll -= 1;
                }
            }
            _ => {}
        }
        return;
    }

    match mouse.kind {
        MouseEventKind::ScrollDown => {
            app.move_down();
        }
        MouseEventKind::ScrollUp => {
            app.move_up();
        }
        _ => {}
    }
}

fn handle_key_event(app: &mut App, key: KeyEvent) {
    // 1. If Confirm Kill Dialog is active
    if app.confirm_kill_target.is_some() {
        match key.code {
            KeyCode::Char('y') | KeyCode::Enter => {
                app.execute_confirmed_kill(false);
            }
            KeyCode::Char('f') | KeyCode::Char('F') => {
                app.execute_confirmed_kill(true);
            }
            KeyCode::Char('n') | KeyCode::Esc => {
                app.confirm_kill_target = None;
                app.set_status("Cancelled process kill");
            }
            _ => {}
        }
        return;
    }

    // 2. If Logs / Inspector modal is active
    if app.logs_modal_open {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Enter => {
                app.logs_modal_open = false;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if app.logs_scroll > 0 {
                    app.logs_scroll -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if app.logs_scroll + 1 < app.logs_content.len() {
                    app.logs_scroll += 1;
                }
            }
            KeyCode::PageUp => {
                app.logs_scroll = app.logs_scroll.saturating_sub(15);
            }
            KeyCode::PageDown => {
                app.logs_scroll =
                    (app.logs_scroll + 15).min(app.logs_content.len().saturating_sub(1));
            }
            KeyCode::Home | KeyCode::Char('g') => {
                app.logs_scroll = 0;
            }
            KeyCode::End | KeyCode::Char('G') => {
                app.logs_scroll = app.logs_content.len().saturating_sub(15);
            }
            _ => {}
        }
        return;
    }

    // 3. If Help modal is active
    if app.help_modal_open {
        match key.code {
            KeyCode::Char('?') | KeyCode::Esc | KeyCode::Char('q') | KeyCode::Enter => {
                app.help_modal_open = false;
            }
            _ => {}
        }
        return;
    }

    // 4. If Search mode is active
    if app.is_searching {
        match key.code {
            KeyCode::Esc | KeyCode::Enter => {
                app.is_searching = false;
            }
            KeyCode::Backspace => {
                app.search_query.pop();
                app.clamp_selections();
            }
            KeyCode::Char(c) => {
                app.search_query.push(c);
                app.clamp_selections();
            }
            _ => {}
        }
        return;
    }

    // 5. Global Hotkeys
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        app.should_quit = true;
        return;
    }

    match key.code {
        KeyCode::Char('q') => {
            app.should_quit = true;
        }
        KeyCode::Tab => {
            app.next_tab();
        }
        KeyCode::BackTab => {
            app.prev_tab();
        }
        KeyCode::Char('1') => {
            app.active_tab = ActiveTab::Ports;
            app.clamp_selections();
        }
        KeyCode::Char('2') => {
            app.active_tab = ActiveTab::Docker;
            app.clamp_selections();
        }
        KeyCode::Char('3') => {
            app.active_tab = ActiveTab::Processes;
            app.clamp_selections();
        }
        KeyCode::Char('4') => {
            app.active_tab = ActiveTab::Services;
            app.clamp_selections();
        }
        KeyCode::Char('5') => {
            app.active_tab = ActiveTab::Storage;
            app.clamp_selections();
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.move_up();
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.move_down();
        }
        KeyCode::PageUp => {
            app.page_up();
        }
        KeyCode::PageDown => {
            app.page_down();
        }
        KeyCode::Home | KeyCode::Char('g') => {
            app.jump_to_top();
        }
        KeyCode::End | KeyCode::Char('G') => {
            app.jump_to_bottom();
        }
        KeyCode::Char('/') => {
            app.is_searching = true;
        }
        KeyCode::Esc => {
            if !app.search_query.is_empty() {
                app.search_query.clear();
                app.clamp_selections();
                app.set_status("Filter cleared");
            }
        }
        KeyCode::Char('r') => {
            app.refresh();
            app.set_status("⚡ Refreshed system state");
        }
        KeyCode::Char('?') => {
            app.help_modal_open = true;
        }

        // Space toggles start/stop on Docker or Services
        KeyCode::Char(' ') => match app.active_tab {
            ActiveTab::Docker => app.toggle_docker_container(),
            ActiveTab::Services => app.toggle_service_state(),
            _ => {}
        },

        // 'x' to kill process or port
        KeyCode::Char('x') | KeyCode::Char('K') => {
            app.request_kill_selected();
        }

        // Tab-specific actions
        KeyCode::Char('o') => {
            if app.active_tab == ActiveTab::Ports {
                app.open_selected_in_browser();
            }
        }

        KeyCode::Char('s') => match app.active_tab {
            ActiveTab::Docker => app.toggle_docker_container(),
            ActiveTab::Services => app.toggle_service_state(),
            _ => app.cycle_sort(),
        },

        KeyCode::Char('S') => {
            app.cycle_sort();
        }

        KeyCode::Char('R') => match app.active_tab {
            ActiveTab::Docker => app.restart_docker_container(),
            ActiveTab::Services => app.restart_selected_service(),
            _ => app.refresh(),
        },

        KeyCode::Char('d') => {
            if app.active_tab == ActiveTab::Services {
                app.disable_now_selected_service();
            }
        }

        KeyCode::Char('e') => {
            if app.active_tab == ActiveTab::Services {
                app.enable_now_selected_service();
            }
        }

        KeyCode::Char('u') => {
            if app.active_tab == ActiveTab::Services {
                app.toggle_services_user_mode();
            } else if app.active_tab == ActiveTab::Storage {
                app.storage_sort = StorageSort::Usage;
                app.set_status("Sorted by Usage %");
                app.clamp_selections();
            }
        }

        KeyCode::Char('t') => {
            if app.active_tab == ActiveTab::Storage {
                app.storage_sort = StorageSort::Total;
                app.set_status("Sorted by Total Size");
                app.clamp_selections();
            }
        }

        KeyCode::Char('l') => match app.active_tab {
            ActiveTab::Docker => app.open_docker_logs(),
            ActiveTab::Services => app.open_service_logs(),
            _ => {}
        },

        KeyCode::Char('a') => {
            if app.active_tab == ActiveTab::Processes {
                app.dev_only_processes = !app.dev_only_processes;
                app.clamp_selections();
                let msg = if app.dev_only_processes {
                    "Showing Dev-only processes"
                } else {
                    "Showing all system processes"
                };
                app.set_status(msg);
            }
        }

        KeyCode::Char('c') => {
            if app.active_tab == ActiveTab::Processes {
                app.process_sort = ProcessSort::Cpu;
                app.clamp_selections();
                app.set_status("Sorted by CPU usage");
            }
        }

        KeyCode::Char('m') => match app.active_tab {
            ActiveTab::Processes => {
                app.process_sort = ProcessSort::Memory;
                app.clamp_selections();
                app.set_status("Sorted by Memory usage");
            }
            ActiveTab::Ports => {
                app.ports_sort = PortsSort::Memory;
                app.set_status("Sorted by Memory usage");
                app.clamp_selections();
            }
            ActiveTab::Storage => {
                app.storage_sort = StorageSort::Mount;
                app.set_status("Sorted by Mount Point");
                app.clamp_selections();
            }
            _ => {}
        },

        KeyCode::Char('p') => match app.active_tab {
            ActiveTab::Processes => {
                app.process_sort = ProcessSort::Pid;
                app.clamp_selections();
                app.set_status("Sorted by PID");
            }
            ActiveTab::Ports => {
                app.ports_sort = PortsSort::Port;
                app.set_status("Sorted by Port number");
                app.clamp_selections();
            }
            _ => {}
        },

        KeyCode::Char('n') => match app.active_tab {
            ActiveTab::Processes => {
                app.process_sort = ProcessSort::Name;
                app.clamp_selections();
                app.set_status("Sorted by Name");
            }
            ActiveTab::Ports => {
                app.ports_sort = PortsSort::Name;
                app.set_status("Sorted by Process Name");
                app.clamp_selections();
            }
            ActiveTab::Docker => {
                app.docker_sort = DockerSort::Name;
                app.set_status("Sorted by Container Name");
                app.clamp_selections();
            }
            _ => {}
        },

        KeyCode::Enter => {
            app.open_process_inspect();
        }

        _ => {}
    }
}
