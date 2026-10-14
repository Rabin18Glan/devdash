mod app;
mod docker;
mod events;
mod ports;
mod services;
mod storage;
mod system;
mod ui;

use clap::{Parser, Subcommand};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::io::{self, stdout};

use crate::app::App;

#[derive(Parser, Debug)]
#[command(name = "devdash", author, version, about = "⚡ Beautiful DevOps, Port & Process Dashboard for Developers", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// List all listening ports and their processes
    Ports,
    /// Kill whatever process is occupying a port (e.g. `devdash kill 3000`)
    Kill {
        /// Port number to free up
        port: u16,
        /// Force kill with SIGKILL (-9)
        #[arg(short, long)]
        force: bool,
    },
    /// List Docker containers and their status
    Docker,
    /// Quick system resource snapshot
    System,
    /// List background systemd services
    Services {
        /// View system services instead of user services
        #[arg(short, long)]
        system: bool,
    },
    /// Disable and immediately stop a service (e.g. `devdash disable openclaw-gateway.service`)
    Disable {
        /// Service unit name
        service: String,
        /// Target system service instead of user service
        #[arg(short, long)]
        system: bool,
    },
    /// Enable and immediately start a service
    Enable {
        /// Service unit name
        service: String,
        /// Target system service instead of user service
        #[arg(short, long)]
        system: bool,
    },
    /// Show storage devices and disk space usage
    Storage,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Ports) => cmd_list_ports(),
        Some(Commands::Kill { port, force }) => cmd_kill_port(port, force),
        Some(Commands::Docker) => cmd_list_docker(),
        Some(Commands::System) => cmd_system_info(),
        Some(Commands::Services { system }) => cmd_list_services(!system),
        Some(Commands::Disable { service, system }) => cmd_disable_service(&service, !system),
        Some(Commands::Enable { service, system }) => cmd_enable_service(&service, !system),
        Some(Commands::Storage) => cmd_list_storage(),
        None => run_tui(),
    }
}

fn cmd_list_ports() -> anyhow::Result<()> {
    let mut sys = system::init_system();
    system::refresh_system(&mut sys);
    let docker_containers = if docker::check_docker_available() {
        docker::list_containers().unwrap_or_default()
    } else {
        Vec::new()
    };
    let port_list = ports::scan_ports(&sys, &docker_containers);

    println!("\x1b[1;36m⚡ DevDash - Active Listening Ports\x1b[0m\n");
    println!(
        "{:<6} {:<8} {:<20} {:<8} {:<24} {:<8} {:<10} {}",
        "PROTO", "PORT", "ADDRESS", "PID", "PROCESS", "CPU%", "MEMORY", "URL / TAG"
    );
    println!("{}", "─".repeat(98));

    for p in port_list {
        let proto_color = if p.protocol == "TCP" {
            "\x1b[32m"
        } else {
            "\x1b[33m"
        };
        let pid_str = p
            .pid
            .map(|n| n.to_string())
            .unwrap_or_else(|| "-".to_string());
        let mem_str = if p.memory_mb > 0.0 {
            format!("{:.1} MB", p.memory_mb)
        } else {
            "-".to_string()
        };
        let cpu_str = if p.cpu_usage > 0.0 {
            format!("{:.1}%", p.cpu_usage)
        } else {
            "-".to_string()
        };
        let url_str = p.url();
        let tag = if p.is_docker_proxy {
            "🐳 Docker Container"
        } else {
            &url_str
        };

        println!(
            "{}{:<6}\x1b[0m \x1b[1;33m:{:<7}\x1b[0m {:<20} \x1b[35m{:<8}\x1b[0m {:<24} {:<8} {:<10} \x1b[36m{}\x1b[0m",
            proto_color,
            p.protocol,
            p.port,
            p.address,
            pid_str,
            p.process_name,
            cpu_str,
            mem_str,
            tag
        );
    }
    println!();
    Ok(())
}

fn cmd_kill_port(port: u16, force: bool) -> anyhow::Result<()> {
    let mut sys = system::init_system();
    system::refresh_system(&mut sys);
    let docker_containers = if docker::check_docker_available() {
        docker::list_containers().unwrap_or_default()
    } else {
        Vec::new()
    };
    let port_list = ports::scan_ports(&sys, &docker_containers);

    let match_opt = port_list.into_iter().find(|p| p.port == port);

    if let Some(target) = match_opt {
        if let Some(pid) = target.pid {
            println!(
                "\x1b[33m⚡ Found process \x1b[1m'{}'\x1b[0m (PID \x1b[35m{}\x1b[0m) on port \x1b[1;33m:{}\x1b[0m",
                target.process_name, pid, port
            );
            ports::kill_process(pid, force)?;
            println!(
                "\x1b[1;32m✔ Successfully terminated PID {} on port :{}\x1b[0m",
                pid, port
            );
        } else if target.is_docker_proxy {
            println!(
                "\x1b[33m⚡ Port :{} is managed by Docker container \x1b[1m'{}'\x1b[0m",
                port, target.process_name
            );
            println!(
                "💡 To stop this port, run: \x1b[1mdevdash\x1b[0m (switch to Docker tab) or \x1b[1mdocker stop <container>\x1b[0m"
            );
        } else {
            eprintln!(
                "\x1b[1;31m✖ Port :{} is in use but PID could not be determined (try running with sudo)\x1b[0m",
                port
            );
        }
    } else {
        println!(
            "\x1b[32m✔ Port :{} is not currently in use by any process.\x1b[0m",
            port
        );
    }

    Ok(())
}

fn cmd_list_docker() -> anyhow::Result<()> {
    if !docker::check_docker_available() {
        eprintln!("\x1b[1;31m✖ Docker daemon is not accessible.\x1b[0m");
        return Ok(());
    }

    let containers = docker::list_containers()?;
    println!("\x1b[1;36m🐳 DevDash - Docker Containers\x1b[0m\n");
    println!(
        "{:<12} {:<24} {:<22} {:<18} {}",
        "STATE", "NAME", "IMAGE", "STATUS", "PORTS"
    );
    println!("{}", "─".repeat(90));

    for c in containers {
        let (state_color, state_label) = if c.is_running() {
            ("\x1b[1;32m", "● RUNNING")
        } else {
            ("\x1b[1;31m", "⏹ STOPPED")
        };

        println!(
            "{}{:<12}\x1b[0m \x1b[1m{:<24}\x1b[0m \x1b[35m{:<22}\x1b[0m {:<18} \x1b[33m{}\x1b[0m",
            state_color, state_label, c.names, c.image, c.status, c.ports
        );
    }
    println!();
    Ok(())
}

fn cmd_system_info() -> anyhow::Result<()> {
    let mut sys = system::init_system();
    system::refresh_system(&mut sys);
    let m = system::get_metrics(&sys);

    println!("\x1b[1;36m📊 DevDash - System Health\x1b[0m\n");
    println!("Host:       \x1b[1m{}\x1b[0m ({})", m.host_name, m.os_name);
    println!("CPU:        \x1b[1;33m{:.1}%\x1b[0m", m.cpu_usage);
    println!(
        "Memory:     \x1b[1;32m{:.1} GB / {:.1} GB ({:.1}%)\x1b[0m",
        m.used_memory_mb as f64 / 1024.0,
        m.total_memory_mb as f64 / 1024.0,
        m.memory_percent
    );
    println!(
        "Swap:       {:.1} GB / {:.1} GB",
        m.used_swap_mb as f64 / 1024.0,
        m.total_swap_mb as f64 / 1024.0
    );
    println!(
        "Load Avg:   {:.2}, {:.2}, {:.2}",
        m.load_avg_one, m.load_avg_five, m.load_avg_fifteen
    );
    println!();
    Ok(())
}

fn cmd_list_services(user_mode: bool) -> anyhow::Result<()> {
    let srv_list = services::list_services(user_mode)?;
    let scope = if user_mode { "User (--user)" } else { "System" };
    println!(
        "\x1b[1;36m⚙ DevDash - Background Services ({})\x1b[0m\n",
        scope
    );
    println!(
        "{:<14} {:<36} {:<12} {}",
        "STATE", "UNIT NAME", "SUBSTATE", "DESCRIPTION"
    );
    println!("{}", "─".repeat(90));

    for s in srv_list {
        let (state_col, label) = if s.is_running() {
            ("\x1b[1;32m", "● RUNNING")
        } else if s.is_failed() {
            ("\x1b[1;31m", "✖ FAILED")
        } else {
            ("\x1b[90m", "⏹ DEAD")
        };

        println!(
            "{}{:<14}\x1b[0m \x1b[1m{:<36}\x1b[0m {:<12} \x1b[90m{}\x1b[0m",
            state_col, label, s.unit, s.sub, s.description
        );
    }
    println!();
    Ok(())
}

fn cmd_disable_service(unit: &str, user_mode: bool) -> anyhow::Result<()> {
    let target = if unit.ends_with(".service") {
        unit.to_string()
    } else {
        format!("{}.service", unit)
    };
    services::disable_now(&target, user_mode)?;
    let scope = if user_mode { "--user" } else { "system" };
    println!(
        "\x1b[1;32m✔ Successfully disabled and stopped {} ({})\x1b[0m",
        target, scope
    );
    Ok(())
}

fn cmd_enable_service(unit: &str, user_mode: bool) -> anyhow::Result<()> {
    let target = if unit.ends_with(".service") {
        unit.to_string()
    } else {
        format!("{}.service", unit)
    };
    services::enable_now(&target, user_mode)?;
    let scope = if user_mode { "--user" } else { "system" };
    println!(
        "\x1b[1;32m✔ Successfully enabled and started {} ({})\x1b[0m",
        target, scope
    );
    Ok(())
}

fn cmd_list_storage() -> anyhow::Result<()> {
    let disks = storage::scan_storage();
    println!("\x1b[1;36m💾 DevDash - Storage Disks & Mounts\x1b[0m\n");
    println!(
        "{:<20} {:<24} {:<10} {:<18} {:<10} {}",
        "MOUNT POINT", "DEVICE", "FS", "USED / TOTAL", "FREE", "USAGE %"
    );
    println!("{}", "─".repeat(90));

    for d in disks {
        let col = if d.usage_percent < 70.0 {
            "\x1b[32m"
        } else if d.usage_percent < 85.0 {
            "\x1b[33m"
        } else {
            "\x1b[31m"
        };

        println!(
            "\x1b[1;36m{:<20}\x1b[0m {:<24} {:<10} {:.1}G / {:.1}G      {:<10.1} {}{:.1}%\x1b[0m",
            d.mount_point,
            d.name,
            d.file_system,
            d.used_gb,
            d.total_gb,
            d.free_gb,
            col,
            d.usage_percent
        );
    }
    println!();
    Ok(())
}

fn run_tui() -> anyhow::Result<()> {
    // Terminal setup
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Set panic hook to restore terminal
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
        original_hook(panic_info);
    }));

    let mut app = App::new();

    // Event loop
    while !app.should_quit {
        // Auto-refresh periodically
        if app.last_refresh.elapsed() >= app.refresh_interval {
            app.refresh();
        }

        terminal.draw(|frame| ui::render(frame, &mut app))?;
        events::handle_events(&mut app)?;
    }

    // Terminal teardown
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
