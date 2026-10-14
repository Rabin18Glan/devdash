use ratatui::widgets::TableState;
use std::time::{Duration, Instant};
use sysinfo::System;

use crate::docker::{self, DockerContainer};
use crate::ports::{self, PortEntry};
use crate::services::{self, SystemdService};
use crate::storage::{self, StorageItem};
use crate::system::{self, ProcessItem, SystemMetrics};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ActiveTab {
    Ports,
    Docker,
    Processes,
    Services,
    Storage,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PortsSort {
    Port,
    Name,
    Memory,
    Pid,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DockerSort {
    Name,
    State,
    Image,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ProcessSort {
    Cpu,
    Memory,
    Pid,
    Name,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ServicesSort {
    Status,
    Name,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum StorageSort {
    Usage,
    Total,
    Mount,
}

#[derive(Clone, Debug)]
pub struct KillTarget {
    pub pid: u32,
    pub name: String,
    pub port_info: Option<String>,
}

pub struct App {
    pub system: System,
    pub metrics: SystemMetrics,

    // Data collections
    pub ports: Vec<PortEntry>,
    pub docker_containers: Vec<DockerContainer>,
    pub docker_available: bool,
    pub processes: Vec<ProcessItem>,
    pub services: Vec<SystemdService>,
    pub services_user_mode: bool,
    pub storage_items: Vec<StorageItem>,

    // Active Navigation
    pub active_tab: ActiveTab,
    pub ports_selected: usize,
    pub docker_selected: usize,
    pub proc_selected: usize,
    pub services_selected: usize,
    pub storage_selected: usize,

    // Stateful table states for smooth scrolling
    pub ports_table_state: TableState,
    pub docker_table_state: TableState,
    pub proc_table_state: TableState,
    pub services_table_state: TableState,
    pub storage_table_state: TableState,

    // Search / Filter
    pub search_query: String,
    pub is_searching: bool,

    // Sort states
    pub ports_sort: PortsSort,
    pub docker_sort: DockerSort,
    pub process_sort: ProcessSort,
    pub services_sort: ServicesSort,
    pub storage_sort: StorageSort,

    pub dev_only_processes: bool,

    // Modals
    pub logs_modal_open: bool,
    pub logs_title: String,
    pub logs_content: Vec<String>,
    pub logs_scroll: usize,

    pub confirm_kill_target: Option<KillTarget>,
    pub help_modal_open: bool,

    // Status / Notification banner
    pub status_message: Option<(String, Instant)>,
    pub last_refresh: Instant,
    pub refresh_interval: Duration,
    pub should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        let mut sys = system::init_system();
        system::refresh_system(&mut sys);
        let metrics = system::get_metrics(&sys);
        let docker_available = docker::check_docker_available();
        let docker_containers = if docker_available {
            docker::list_containers().unwrap_or_default()
        } else {
            Vec::new()
        };
        let ports = ports::scan_ports(&sys, &docker_containers);
        let processes = system::scan_processes(&sys);
        let services = services::list_services(true).unwrap_or_default();
        let storage_items = storage::scan_storage();

        let mut ports_table_state = TableState::default();
        if !ports.is_empty() {
            ports_table_state.select(Some(0));
        }

        let mut docker_table_state = TableState::default();
        if !docker_containers.is_empty() {
            docker_table_state.select(Some(0));
        }

        let mut proc_table_state = TableState::default();
        if !processes.is_empty() {
            proc_table_state.select(Some(0));
        }

        let mut services_table_state = TableState::default();
        if !services.is_empty() {
            services_table_state.select(Some(0));
        }

        let mut storage_table_state = TableState::default();
        if !storage_items.is_empty() {
            storage_table_state.select(Some(0));
        }

        Self {
            system: sys,
            metrics,
            ports,
            docker_containers,
            docker_available,
            processes,
            services,
            services_user_mode: true,
            storage_items,
            active_tab: ActiveTab::Ports,
            ports_selected: 0,
            docker_selected: 0,
            proc_selected: 0,
            services_selected: 0,
            storage_selected: 0,
            ports_table_state,
            docker_table_state,
            proc_table_state,
            services_table_state,
            storage_table_state,
            search_query: String::new(),
            is_searching: false,
            ports_sort: PortsSort::Port,
            docker_sort: DockerSort::State,
            process_sort: ProcessSort::Cpu,
            services_sort: ServicesSort::Status,
            storage_sort: StorageSort::Total,
            dev_only_processes: true,
            logs_modal_open: false,
            logs_title: String::new(),
            logs_content: Vec::new(),
            logs_scroll: 0,
            confirm_kill_target: None,
            help_modal_open: false,
            status_message: Some((
                "⚡ DevDash ready. Press '?' for shortcuts.".to_string(),
                Instant::now(),
            )),
            last_refresh: Instant::now(),
            refresh_interval: Duration::from_secs(2),
            should_quit: false,
        }
    }

    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = Some((msg.into(), Instant::now()));
    }

    pub fn refresh(&mut self) {
        system::refresh_system(&mut self.system);
        self.metrics = system::get_metrics(&self.system);
        self.docker_available = docker::check_docker_available();
        if self.docker_available {
            if let Ok(c) = docker::list_containers() {
                self.docker_containers = c;
            }
        } else {
            self.docker_containers.clear();
        }
        self.ports = ports::scan_ports(&self.system, &self.docker_containers);
        self.processes = system::scan_processes(&self.system);
        if let Ok(srv) = services::list_services(self.services_user_mode) {
            self.services = srv;
        }
        self.storage_items = storage::scan_storage();

        self.last_refresh = Instant::now();
        self.clamp_selections();
    }

    pub fn filtered_ports(&self) -> Vec<PortEntry> {
        let q = self.search_query.trim().to_lowercase();
        let mut list: Vec<PortEntry> = self.ports
            .iter()
            .filter(|p| {
                if q.is_empty() {
                    return true;
                }
                p.port.to_string().contains(&q)
                    || p.process_name.to_lowercase().contains(&q)
                    || p.cmdline.to_lowercase().contains(&q)
                    || p.address.contains(&q)
                    || p.protocol.to_lowercase().contains(&q)
            })
            .cloned()
            .collect();

        match self.ports_sort {
            PortsSort::Port => list.sort_by_key(|p| p.port),
            PortsSort::Name => list.sort_by(|a, b| a.process_name.to_lowercase().cmp(&b.process_name.to_lowercase())),
            PortsSort::Memory => list.sort_by(|a, b| b.memory_mb.partial_cmp(&a.memory_mb).unwrap_or(std::cmp::Ordering::Equal)),
            PortsSort::Pid => list.sort_by_key(|p| p.pid.unwrap_or(0)),
        }

        list
    }

    pub fn filtered_docker(&self) -> Vec<DockerContainer> {
        let q = self.search_query.trim().to_lowercase();
        let mut list: Vec<DockerContainer> = self.docker_containers
            .iter()
            .filter(|c| {
                if q.is_empty() {
                    return true;
                }
                c.names.to_lowercase().contains(&q)
                    || c.image.to_lowercase().contains(&q)
                    || c.status.to_lowercase().contains(&q)
                    || c.ports.to_lowercase().contains(&q)
                    || c.id.to_lowercase().contains(&q)
            })
            .cloned()
            .collect();

        match self.docker_sort {
            DockerSort::Name => list.sort_by(|a, b| a.names.to_lowercase().cmp(&b.names.to_lowercase())),
            DockerSort::State => list.sort_by(|a, b| {
                let a_r = if a.is_running() { 0 } else { 1 };
                let b_r = if b.is_running() { 0 } else { 1 };
                a_r.cmp(&b_r).then_with(|| a.names.cmp(&b.names))
            }),
            DockerSort::Image => list.sort_by(|a, b| a.image.to_lowercase().cmp(&b.image.to_lowercase())),
        }

        list
    }

    pub fn filtered_processes(&self) -> Vec<ProcessItem> {
        let q = self.search_query.trim().to_lowercase();
        let mut list: Vec<ProcessItem> = self.processes
            .iter()
            .filter(|p| {
                if self.dev_only_processes && !p.is_dev_related {
                    return false;
                }
                if q.is_empty() {
                    return true;
                }
                p.pid.to_string().contains(&q)
                    || p.name.to_lowercase().contains(&q)
                    || p.cmdline.to_lowercase().contains(&q)
            })
            .cloned()
            .collect();

        match self.process_sort {
            ProcessSort::Cpu => list.sort_by(|a, b| b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap_or(std::cmp::Ordering::Equal)),
            ProcessSort::Memory => list.sort_by(|a, b| b.memory_mb.partial_cmp(&a.memory_mb).unwrap_or(std::cmp::Ordering::Equal)),
            ProcessSort::Pid => list.sort_by_key(|p| p.pid),
            ProcessSort::Name => list.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase())),
        }

        list
    }

    pub fn filtered_services(&self) -> Vec<SystemdService> {
        let q = self.search_query.trim().to_lowercase();
        let mut list: Vec<SystemdService> = self.services
            .iter()
            .filter(|s| {
                if q.is_empty() {
                    return true;
                }
                s.unit.to_lowercase().contains(&q)
                    || s.description.to_lowercase().contains(&q)
                    || s.active.to_lowercase().contains(&q)
                    || s.sub.to_lowercase().contains(&q)
            })
            .cloned()
            .collect();

        match self.services_sort {
            ServicesSort::Status => list.sort_by(|a, b| {
                let a_score = if a.is_running() { 0 } else if a.is_failed() { 1 } else { 2 };
                let b_score = if b.is_running() { 0 } else if b.is_failed() { 1 } else { 2 };
                a_score.cmp(&b_score).then_with(|| a.unit.cmp(&b.unit))
            }),
            ServicesSort::Name => list.sort_by(|a, b| a.unit.to_lowercase().cmp(&b.unit.to_lowercase())),
        }

        list
    }

    pub fn filtered_storage(&self) -> Vec<StorageItem> {
        let q = self.search_query.trim().to_lowercase();
        let mut list: Vec<StorageItem> = self.storage_items
            .iter()
            .filter(|d| {
                if q.is_empty() {
                    return true;
                }
                d.mount_point.to_lowercase().contains(&q)
                    || d.name.to_lowercase().contains(&q)
                    || d.file_system.to_lowercase().contains(&q)
            })
            .cloned()
            .collect();

        match self.storage_sort {
            StorageSort::Total => list.sort_by(|a, b| b.total_gb.partial_cmp(&a.total_gb).unwrap_or(std::cmp::Ordering::Equal)),
            StorageSort::Usage => list.sort_by(|a, b| b.usage_percent.partial_cmp(&a.usage_percent).unwrap_or(std::cmp::Ordering::Equal)),
            StorageSort::Mount => list.sort_by(|a, b| a.mount_point.cmp(&b.mount_point)),
        }

        list
    }

    pub fn next_tab(&mut self) {
        self.active_tab = match self.active_tab {
            ActiveTab::Ports => ActiveTab::Docker,
            ActiveTab::Docker => ActiveTab::Processes,
            ActiveTab::Processes => ActiveTab::Services,
            ActiveTab::Services => ActiveTab::Storage,
            ActiveTab::Storage => ActiveTab::Ports,
        };
        self.clamp_selections();
    }

    pub fn prev_tab(&mut self) {
        self.active_tab = match self.active_tab {
            ActiveTab::Ports => ActiveTab::Storage,
            ActiveTab::Docker => ActiveTab::Ports,
            ActiveTab::Processes => ActiveTab::Docker,
            ActiveTab::Services => ActiveTab::Processes,
            ActiveTab::Storage => ActiveTab::Services,
        };
        self.clamp_selections();
    }

    pub fn move_up(&mut self) {
        match self.active_tab {
            ActiveTab::Ports => {
                if self.ports_selected > 0 {
                    self.ports_selected -= 1;
                }
            }
            ActiveTab::Docker => {
                if self.docker_selected > 0 {
                    self.docker_selected -= 1;
                }
            }
            ActiveTab::Processes => {
                if self.proc_selected > 0 {
                    self.proc_selected -= 1;
                }
            }
            ActiveTab::Services => {
                if self.services_selected > 0 {
                    self.services_selected -= 1;
                }
            }
            ActiveTab::Storage => {
                if self.storage_selected > 0 {
                    self.storage_selected -= 1;
                }
            }
        }
        self.clamp_selections();
    }

    pub fn move_down(&mut self) {
        match self.active_tab {
            ActiveTab::Ports => {
                let count = self.filtered_ports().len();
                if count > 0 && self.ports_selected + 1 < count {
                    self.ports_selected += 1;
                }
            }
            ActiveTab::Docker => {
                let count = self.filtered_docker().len();
                if count > 0 && self.docker_selected + 1 < count {
                    self.docker_selected += 1;
                }
            }
            ActiveTab::Processes => {
                let count = self.filtered_processes().len();
                if count > 0 && self.proc_selected + 1 < count {
                    self.proc_selected += 1;
                }
            }
            ActiveTab::Services => {
                let count = self.filtered_services().len();
                if count > 0 && self.services_selected + 1 < count {
                    self.services_selected += 1;
                }
            }
            ActiveTab::Storage => {
                let count = self.filtered_storage().len();
                if count > 0 && self.storage_selected + 1 < count {
                    self.storage_selected += 1;
                }
            }
        }
        self.clamp_selections();
    }

    pub fn page_up(&mut self) {
        match self.active_tab {
            ActiveTab::Ports => self.ports_selected = self.ports_selected.saturating_sub(10),
            ActiveTab::Docker => self.docker_selected = self.docker_selected.saturating_sub(10),
            ActiveTab::Processes => self.proc_selected = self.proc_selected.saturating_sub(10),
            ActiveTab::Services => self.services_selected = self.services_selected.saturating_sub(10),
            ActiveTab::Storage => self.storage_selected = self.storage_selected.saturating_sub(10),
        }
        self.clamp_selections();
    }

    pub fn page_down(&mut self) {
        match self.active_tab {
            ActiveTab::Ports => {
                let count = self.filtered_ports().len();
                if count > 0 {
                    self.ports_selected = (self.ports_selected + 10).min(count - 1);
                }
            }
            ActiveTab::Docker => {
                let count = self.filtered_docker().len();
                if count > 0 {
                    self.docker_selected = (self.docker_selected + 10).min(count - 1);
                }
            }
            ActiveTab::Processes => {
                let count = self.filtered_processes().len();
                if count > 0 {
                    self.proc_selected = (self.proc_selected + 10).min(count - 1);
                }
            }
            ActiveTab::Services => {
                let count = self.filtered_services().len();
                if count > 0 {
                    self.services_selected = (self.services_selected + 10).min(count - 1);
                }
            }
            ActiveTab::Storage => {
                let count = self.filtered_storage().len();
                if count > 0 {
                    self.storage_selected = (self.storage_selected + 10).min(count - 1);
                }
            }
        }
        self.clamp_selections();
    }

    pub fn jump_to_top(&mut self) {
        match self.active_tab {
            ActiveTab::Ports => self.ports_selected = 0,
            ActiveTab::Docker => self.docker_selected = 0,
            ActiveTab::Processes => self.proc_selected = 0,
            ActiveTab::Services => self.services_selected = 0,
            ActiveTab::Storage => self.storage_selected = 0,
        }
        self.clamp_selections();
    }

    pub fn jump_to_bottom(&mut self) {
        match self.active_tab {
            ActiveTab::Ports => {
                let count = self.filtered_ports().len();
                if count > 0 { self.ports_selected = count - 1; }
            }
            ActiveTab::Docker => {
                let count = self.filtered_docker().len();
                if count > 0 { self.docker_selected = count - 1; }
            }
            ActiveTab::Processes => {
                let count = self.filtered_processes().len();
                if count > 0 { self.proc_selected = count - 1; }
            }
            ActiveTab::Services => {
                let count = self.filtered_services().len();
                if count > 0 { self.services_selected = count - 1; }
            }
            ActiveTab::Storage => {
                let count = self.filtered_storage().len();
                if count > 0 { self.storage_selected = count - 1; }
            }
        }
        self.clamp_selections();
    }

    pub fn clamp_selections(&mut self) {
        let ports_count = self.filtered_ports().len();
        if ports_count == 0 {
            self.ports_selected = 0;
            self.ports_table_state.select(None);
        } else {
            if self.ports_selected >= ports_count { self.ports_selected = ports_count - 1; }
            self.ports_table_state.select(Some(self.ports_selected));
        }

        let docker_count = self.filtered_docker().len();
        if docker_count == 0 {
            self.docker_selected = 0;
            self.docker_table_state.select(None);
        } else {
            if self.docker_selected >= docker_count { self.docker_selected = docker_count - 1; }
            self.docker_table_state.select(Some(self.docker_selected));
        }

        let proc_count = self.filtered_processes().len();
        if proc_count == 0 {
            self.proc_selected = 0;
            self.proc_table_state.select(None);
        } else {
            if self.proc_selected >= proc_count { self.proc_selected = proc_count - 1; }
            self.proc_table_state.select(Some(self.proc_selected));
        }

        let srv_count = self.filtered_services().len();
        if srv_count == 0 {
            self.services_selected = 0;
            self.services_table_state.select(None);
        } else {
            if self.services_selected >= srv_count { self.services_selected = srv_count - 1; }
            self.services_table_state.select(Some(self.services_selected));
        }

        let st_count = self.filtered_storage().len();
        if st_count == 0 {
            self.storage_selected = 0;
            self.storage_table_state.select(None);
        } else {
            if self.storage_selected >= st_count { self.storage_selected = st_count - 1; }
            self.storage_table_state.select(Some(self.storage_selected));
        }
    }

    pub fn cycle_sort(&mut self) {
        match self.active_tab {
            ActiveTab::Ports => {
                self.ports_sort = match self.ports_sort {
                    PortsSort::Port => PortsSort::Name,
                    PortsSort::Name => PortsSort::Memory,
                    PortsSort::Memory => PortsSort::Pid,
                    PortsSort::Pid => PortsSort::Port,
                };
                self.set_status(format!("Sorted by {:?}", self.ports_sort));
            }
            ActiveTab::Docker => {
                self.docker_sort = match self.docker_sort {
                    DockerSort::State => DockerSort::Name,
                    DockerSort::Name => DockerSort::Image,
                    DockerSort::Image => DockerSort::State,
                };
                self.set_status(format!("Sorted by {:?}", self.docker_sort));
            }
            ActiveTab::Processes => {
                self.process_sort = match self.process_sort {
                    ProcessSort::Cpu => ProcessSort::Memory,
                    ProcessSort::Memory => ProcessSort::Pid,
                    ProcessSort::Pid => ProcessSort::Name,
                    ProcessSort::Name => ProcessSort::Cpu,
                };
                self.set_status(format!("Sorted by {:?}", self.process_sort));
            }
            ActiveTab::Services => {
                self.services_sort = match self.services_sort {
                    ServicesSort::Status => ServicesSort::Name,
                    ServicesSort::Name => ServicesSort::Status,
                };
                self.set_status(format!("Sorted by {:?}", self.services_sort));
            }
            ActiveTab::Storage => {
                self.storage_sort = match self.storage_sort {
                    StorageSort::Total => StorageSort::Usage,
                    StorageSort::Usage => StorageSort::Mount,
                    StorageSort::Mount => StorageSort::Total,
                };
                self.set_status(format!("Sorted by {:?}", self.storage_sort));
            }
        }
        self.clamp_selections();
    }

    pub fn toggle_services_user_mode(&mut self) {
        self.services_user_mode = !self.services_user_mode;
        if let Ok(srv) = services::list_services(self.services_user_mode) {
            self.services = srv;
        }
        let label = if self.services_user_mode { "User services (systemctl --user)" } else { "System services (systemctl)" };
        self.set_status(format!("Switched to {}", label));
        self.clamp_selections();
    }

    pub fn toggle_service_state(&mut self) {
        let list = self.filtered_services();
        if let Some(srv) = list.get(self.services_selected) {
            let unit = srv.unit.clone();
            let is_user = self.services_user_mode;
            let is_running = srv.is_running();

            let res = if is_running {
                services::stop_service(&unit, is_user)
            } else {
                services::start_service(&unit, is_user)
            };

            match res {
                Ok(_) => {
                    let action = if is_running { "Stopped" } else { "Started" };
                    self.set_status(format!("✔ {} service '{}'", action, unit));
                    self.refresh();
                }
                Err(e) => {
                    self.set_status(format!("✖ Failed: {}", e));
                }
            }
        }
    }

    pub fn restart_selected_service(&mut self) {
        let list = self.filtered_services();
        if let Some(srv) = list.get(self.services_selected) {
            let unit = srv.unit.clone();
            let is_user = self.services_user_mode;
            match services::restart_service(&unit, is_user) {
                Ok(_) => {
                    self.set_status(format!("✔ Restarted service '{}'", unit));
                    self.refresh();
                }
                Err(e) => {
                    self.set_status(format!("✖ Failed to restart: {}", e));
                }
            }
        }
    }

    pub fn disable_now_selected_service(&mut self) {
        let list = self.filtered_services();
        if let Some(srv) = list.get(self.services_selected) {
            let unit = srv.unit.clone();
            let is_user = self.services_user_mode;
            match services::disable_now(&unit, is_user) {
                Ok(_) => {
                    self.set_status(format!("✔ Disabled & stopped service '{}'", unit));
                    self.refresh();
                }
                Err(e) => {
                    self.set_status(format!("✖ Failed to disable --now: {}", e));
                }
            }
        }
    }

    pub fn enable_now_selected_service(&mut self) {
        let list = self.filtered_services();
        if let Some(srv) = list.get(self.services_selected) {
            let unit = srv.unit.clone();
            let is_user = self.services_user_mode;
            match services::enable_now(&unit, is_user) {
                Ok(_) => {
                    self.set_status(format!("✔ Enabled & started service '{}'", unit));
                    self.refresh();
                }
                Err(e) => {
                    self.set_status(format!("✖ Failed to enable --now: {}", e));
                }
            }
        }
    }

    pub fn open_service_logs(&mut self) {
        let list = self.filtered_services();
        if let Some(srv) = list.get(self.services_selected) {
            let unit = srv.unit.clone();
            let is_user = self.services_user_mode;
            match services::get_service_logs(&unit, is_user, 150) {
                Ok(logs) => {
                    self.logs_title = format!("📋 Journal Logs: {}", unit);
                    self.logs_content = logs.lines().map(|s| s.to_string()).collect();
                    self.logs_scroll = self.logs_content.len().saturating_sub(20);
                    self.logs_modal_open = true;
                }
                Err(e) => {
                    self.set_status(format!("✖ Failed to read journal: {}", e));
                }
            }
        }
    }

    pub fn open_selected_in_browser(&mut self) {
        if self.active_tab == ActiveTab::Ports {
            let list = self.filtered_ports();
            if let Some(entry) = list.get(self.ports_selected) {
                let url = entry.url();
                match ports::open_in_browser(&url) {
                    Ok(_) => self.set_status(format!("🌐 Opened {} in browser", url)),
                    Err(e) => self.set_status(format!("✖ Failed to open browser: {}", e)),
                }
            }
        }
    }

    pub fn request_kill_selected(&mut self) {
        match self.active_tab {
            ActiveTab::Ports => {
                let list = self.filtered_ports();
                if let Some(entry) = list.get(self.ports_selected) {
                    if let Some(pid) = entry.pid {
                        self.confirm_kill_target = Some(KillTarget {
                            pid,
                            name: entry.process_name.clone(),
                            port_info: Some(format!("Port :{}", entry.port)),
                        });
                    } else {
                        self.set_status(format!(
                            "✖ No PID found for port :{} (may need root or Docker stop)",
                            entry.port
                         ));
                    }
                }
            }
            ActiveTab::Processes => {
                let list = self.filtered_processes();
                if let Some(item) = list.get(self.proc_selected) {
                    self.confirm_kill_target = Some(KillTarget {
                        pid: item.pid,
                        name: item.name.clone(),
                        port_info: None,
                    });
                }
            }
            ActiveTab::Docker => {
                self.toggle_docker_container();
            }
            ActiveTab::Services => {
                self.toggle_service_state();
            }
            ActiveTab::Storage => {}
        }
    }

    pub fn execute_confirmed_kill(&mut self, force: bool) {
        if let Some(target) = self.confirm_kill_target.take() {
            match ports::kill_process(target.pid, force) {
                Ok(_) => {
                    self.set_status(format!(
                        "✔ Terminated '{}' (PID {})",
                        target.name, target.pid
                    ));
                    self.refresh();
                }
                Err(e) => {
                    self.set_status(format!("✖ Failed to kill PID {}: {}", target.pid, e));
                }
            }
        }
    }

    pub fn toggle_docker_container(&mut self) {
        if !self.docker_available {
            self.set_status("✖ Docker daemon is not accessible");
            return;
        }
        let list = self.filtered_docker();
        if let Some(c) = list.get(self.docker_selected) {
            let id = c.id.clone();
            let name = c.names.clone();
            let is_running = c.is_running();

            let res = if is_running {
                docker::stop_container(&id)
            } else {
                docker::start_container(&id)
            };

            match res {
                Ok(_) => {
                    let action = if is_running { "Stopped" } else { "Started" };
                    self.set_status(format!("✔ {} container '{}'", action, name));
                    self.refresh();
                }
                Err(e) => {
                    self.set_status(format!("✖ Docker action failed: {}", e));
                }
            }
        }
    }

    pub fn restart_docker_container(&mut self) {
        if !self.docker_available {
            self.set_status("✖ Docker daemon is not accessible");
            return;
        }
        let list = self.filtered_docker();
        if let Some(c) = list.get(self.docker_selected) {
            let id = c.id.clone();
            let name = c.names.clone();
            match docker::restart_container(&id) {
                Ok(_) => {
                    self.set_status(format!("✔ Restarted container '{}'", name));
                    self.refresh();
                }
                Err(e) => {
                    self.set_status(format!("✖ Docker restart failed: {}", e));
                }
            }
        }
    }

    pub fn open_docker_logs(&mut self) {
        if !self.docker_available {
            self.set_status("✖ Docker daemon is not accessible");
            return;
        }
        let list = self.filtered_docker();
        if let Some(c) = list.get(self.docker_selected) {
            let id = c.id.clone();
            let name = c.names.clone();
            match docker::get_container_logs(&id, 300) {
                Ok(logs) => {
                    self.logs_title = format!(
                        "📋 Container Logs: {} ({})",
                        name,
                          &id[..std::cmp::min(12, id.len())]
                    );
                    self.logs_content = logs.lines().map(|s| s.to_string()).collect();
                    self.logs_scroll = self.logs_content.len().saturating_sub(20);
                    self.logs_modal_open = true;
                }
                Err(e) => {
                    self.set_status(format!("✖ Failed to read logs: {}", e));
                }
            }
        }
    }

    pub fn open_process_inspect(&mut self) {
        let (title, content) = match self.active_tab {
            ActiveTab::Ports => {
                let list = self.filtered_ports();
                if let Some(p) = list.get(self.ports_selected) {
                    let details = vec![
                        format!("Port:          :{}", p.port),
                        format!("Protocol:      {}", p.protocol),
                        format!("Bind Address:  {}", p.address),
                        format!(
                            "PID:           {}",
                            p.pid
                                .map(|n| n.to_string())
                                 .unwrap_or_else(|| "N/A".to_string())
                        ),
                        format!("Process Name:  {}", p.process_name),
                        format!("CPU Usage:     {:.1}%", p.cpu_usage),
                        format!("Memory Usage:  {:.2} MB", p.memory_mb),
                        format!(
                            "Docker Proxy:  {}",
                             if p.is_docker_proxy { "Yes" } else { "No" }
                        ),
                        format!("URL:           {}", p.url()),
                        "".to_string(),
                        "Full Command Line:".to_string(),
                        if p.cmdline.is_empty() {
                             "(no arguments / system process)".to_string()
                        } else {
                            p.cmdline.clone()
                           },
                    ];
                    (format!("🔍 Port Details: :{}", p.port), details)
                } else {
                    return;
                }
            }
            ActiveTab::Processes => {
                let list = self.filtered_processes();
                if let Some(p) = list.get(self.proc_selected) {
                    let details = vec![
                        format!("PID:           {}", p.pid),
                        format!("Process Name:  {}", p.name),
                        format!("CPU Usage:     {:.1}%", p.cpu_usage),
                        format!("Memory Usage:  {:.2} MB", p.memory_mb),
                        format!(
                            "Dev Keyword:   {}",
                            if p.is_dev_related {
                                "Yes (Dev Service)"
                            } else {
                                "No"
                            }
                             ),
                        "".to_string(),
                        "Full Command Line:".to_string(),
                        if p.cmdline.is_empty() {
                             "(no arguments)".to_string()
                        } else {
                            p.cmdline.clone()
                           },
                    ];
                    (
                        format!("🔍 Process Details: {} (PID {})", p.name, p.pid),
                        details,
                     )
                } else {
                    return;
                }
            }
            ActiveTab::Docker => {
                self.open_docker_logs();
                return;
            }
            ActiveTab::Services => {
                let list = self.filtered_services();
                if let Some(s) = list.get(self.services_selected) {
                    let is_user = self.services_user_mode;
                    let text = services::get_service_status_text(&s.unit, is_user).unwrap_or_default();
                    self.logs_title = format!("⚙ Service Status: {}", s.unit);
                    self.logs_content = text.lines().map(|l| l.to_string()).collect();
                    self.logs_scroll = 0;
                    self.logs_modal_open = true;
                }
                return;
            }
            ActiveTab::Storage => {
                let list = self.filtered_storage();
                if let Some(d) = list.get(self.storage_selected) {
                    let details = vec![
                        format!("Mount Point:   {}", d.mount_point),
                        format!("Device Name:   {}", d.name),
                        format!("Filesystem:    {}", d.file_system),
                        format!("Total Space:   {:.2} GB", d.total_gb),
                        format!("Used Space:    {:.2} GB ({:.1}%)", d.used_gb, d.usage_percent),
                        format!("Free Space:    {:.2} GB", d.free_gb),
                        format!("Is Removable:  {}", if d.is_removable { "Yes (USB / External)" } else { "No" }),
                    ];
                    (format!("💾 Storage Details: {}", d.mount_point), details)
                } else {
                    return;
                }
            }
        };

        self.logs_title = title;
        self.logs_content = content;
        self.logs_scroll = 0;
        self.logs_modal_open = true;
    }
}
