use sysinfo::{CpuRefreshKind, MemoryRefreshKind, ProcessRefreshKind, RefreshKind, System};

#[derive(Clone, Debug)]
pub struct SystemMetrics {
    pub cpu_usage: f32,
    pub used_memory_mb: u64,
    pub total_memory_mb: u64,
    pub memory_percent: f32,
    pub used_swap_mb: u64,
    pub total_swap_mb: u64,
    pub load_avg_one: f64,
    pub load_avg_five: f64,
    pub load_avg_fifteen: f64,
    pub host_name: String,
    pub os_name: String,
    pub uptime_secs: u64,
}

#[derive(Clone, Debug)]
pub struct ProcessItem {
    pub pid: u32,
    pub name: String,
    pub cpu_usage: f32,
    pub memory_mb: f64,
    pub cmdline: String,
    pub is_dev_related: bool,
}

pub fn init_system() -> System {
    let refresh_kind = RefreshKind::nothing()
        .with_cpu(CpuRefreshKind::everything())
        .with_memory(MemoryRefreshKind::everything())
        .with_processes(ProcessRefreshKind::everything());
    System::new_with_specifics(refresh_kind)
}

pub fn refresh_system(sys: &mut System) {
    sys.refresh_cpu_all();
    sys.refresh_memory();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
}

pub fn get_metrics(sys: &System) -> SystemMetrics {
    let cpu_usage = sys.global_cpu_usage();
    let used_memory_mb = sys.used_memory() / (1024 * 1024);
    let total_memory_mb = sys.total_memory() / (1024 * 1024);
    let memory_percent = if total_memory_mb > 0 {
        (used_memory_mb as f32 / total_memory_mb as f32) * 100.0
    } else {
        0.0
    };

    let used_swap_mb = sys.used_swap() / (1024 * 1024);
    let total_swap_mb = sys.total_swap() / (1024 * 1024);

    let load_avg = System::load_average();

    let host_name = System::host_name().unwrap_or_else(|| "localhost".to_string());
    let os_name = System::long_os_version().unwrap_or_else(|| "Linux".to_string());
    let uptime_secs = System::uptime();

    SystemMetrics {
        cpu_usage,
        used_memory_mb,
        total_memory_mb,
        memory_percent,
        used_swap_mb,
        total_swap_mb,
        load_avg_one: load_avg.one,
        load_avg_five: load_avg.five,
        load_avg_fifteen: load_avg.fifteen,
        host_name,
        os_name,
        uptime_secs,
    }
}

pub fn scan_processes(sys: &System) -> Vec<ProcessItem> {
    let mut items = Vec::new();

    let dev_keywords = [
        "node",
        "npm",
        "npx",
        "yarn",
        "pnpm",
        "bun",
        "deno",
        "python",
        "python3",
        "flask",
        "django",
        "uvicorn",
        "gunicorn",
        "celery",
        "cargo",
        "rustc",
        "rust-analyzer",
        "go",
        "gopls",
        "air",
        "docker",
        "dockerd",
        "containerd",
        "postgres",
        "mysql",
        "mariadb",
        "redis-server",
        "mongod",
        "nginx",
        "caddy",
        "apache2",
        "httpd",
        "vite",
        "next-server",
        "webpack",
        "esbuild",
        "turbo",
        "dx",
        "live-server",
        "agy_acp",
        "code",
        "nvim",
    ];

    for (pid, proc_entry) in sys.processes() {
        let name = proc_entry.name().to_string_lossy().to_string();
        let cmdline = proc_entry
            .cmd()
            .iter()
            .map(|s| s.to_string_lossy().to_string())
            .collect::<Vec<_>>()
            .join(" ");

        let name_lower = name.to_lowercase();
        let cmd_lower = cmdline.to_lowercase();

        let is_dev = dev_keywords
            .iter()
            .any(|k| name_lower.contains(k) || cmd_lower.contains(k));

        let memory_mb = proc_entry.memory() as f64 / (1024.0 * 1024.0);
        let cpu_usage = proc_entry.cpu_usage();

        items.push(ProcessItem {
            pid: pid.as_u32(),
            name,
            cpu_usage,
            memory_mb,
            cmdline,
            is_dev_related: is_dev,
        });
    }

    // Default sort by CPU descending
    items.sort_by(|a, b| {
        b.cpu_usage
            .partial_cmp(&a.cpu_usage)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    items
}
