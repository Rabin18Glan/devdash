use crate::docker::DockerContainer;
use std::process::Command;
use sysinfo::{Pid, System};

#[derive(Clone, Debug, PartialEq)]
pub struct PortEntry {
    pub protocol: String,
    pub port: u16,
    pub address: String,
    pub pid: Option<u32>,
    pub process_name: String,
    pub cmdline: String,
    pub memory_mb: f64,
    pub cpu_usage: f32,
    pub is_docker_proxy: bool,
}

impl PortEntry {
    #[allow(dead_code)]
    pub fn display_target(&self) -> String {
        format!("{}:{}", self.address, self.port)
    }

    pub fn url(&self) -> String {
        let host = if self.address == "0.0.0.0" || self.address == "::" || self.address == "[::]" {
            "localhost"
        } else if self.address.starts_with("127.")
            || self.address == "::1"
            || self.address == "[::1]"
        {
            "localhost"
        } else {
            &self.address
        };
        format!("http://{}:{}", host, self.port)
    }
}

pub fn scan_ports(sys: &System, docker_containers: &[DockerContainer]) -> Vec<PortEntry> {
    let mut entries = Vec::new();

    // 1. Try lsof first (very accurate for TCP listeners and associated PIDs)
    if let Ok(output) = Command::new("lsof").args(["-i", "-P", "-n"]).output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if !line.contains("LISTEN") && !line.contains("UDP") {
                    continue;
                }
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() < 9 {
                    continue;
                }

                let proc_name_raw = parts[0];
                let pid_parsed = parts[1].parse::<u32>().ok();
                let proto = parts[7].to_uppercase();
                let addr_port = parts[8];

                if let Some((addr, port)) = parse_address_port(addr_port) {
                    let (cmd, mem, cpu, name) = resolve_proc_info(sys, pid_parsed, proc_name_raw);
                    let is_docker = name.contains("docker-proxy") || cmd.contains("docker-proxy");

                    if !entries
                        .iter()
                        .any(|e: &PortEntry| e.port == port && e.protocol == proto)
                    {
                        entries.push(PortEntry {
                            protocol: proto,
                            port,
                            address: addr,
                            pid: pid_parsed,
                            process_name: name,
                            cmdline: cmd,
                            memory_mb: mem,
                            cpu_usage: cpu,
                            is_docker_proxy: is_docker,
                        });
                    }
                }
            }
        }
    }

    // 2. Supplement with ss -tulpnH in case lsof missed some
    if let Ok(output) = Command::new("ss").args(["-tulpnH"]).output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() < 5 {
                    continue;
                }

                let proto = parts[0].to_uppercase();
                let local_addr = parts[4];

                if let Some((addr, port)) = parse_address_port(local_addr) {
                    if !entries
                        .iter()
                        .any(|e| e.port == port && e.protocol == proto)
                    {
                        let (pid, proc_name) = parse_ss_users(line);
                        let (cmd, mem, cpu, name) = resolve_proc_info(sys, pid, &proc_name);
                        let is_docker =
                            name.contains("docker-proxy") || cmd.contains("docker-proxy");

                        entries.push(PortEntry {
                            protocol: proto,
                            port,
                            address: addr,
                            pid,
                            process_name: name,
                            cmdline: cmd,
                            memory_mb: mem,
                            cpu_usage: cpu,
                            is_docker_proxy: is_docker,
                        });
                    }
                }
            }
        }
    }

    // 3. Enrich entries using Docker container port mappings
    for container in docker_containers {
        // e.g. "0.0.0.0:3306->3306/tcp, [::]:3306->3306/tcp"
        for part in container.ports.split(',') {
            if let Some(arrow_idx) = part.find("->") {
                let host_side = &part[..arrow_idx].trim();
                if let Some(colon) = host_side.rfind(':') {
                    if let Ok(host_port) = host_side[colon + 1..].parse::<u16>() {
                        for entry in entries.iter_mut() {
                            if entry.port == host_port {
                                entry.is_docker_proxy = true;
                                if entry.process_name == "unknown"
                                    || entry.process_name.is_empty()
                                    || entry.process_name == "docker-proxy"
                                {
                                    entry.process_name = format!("🐳 {}", container.names);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    entries.sort_by_key(|e| e.port);
    entries
}

fn parse_address_port(addr_str: &str) -> Option<(String, u16)> {
    let clean = addr_str.trim().trim_end_matches("(LISTEN)").trim();

    if let Some(bracket_end) = clean.rfind(']') {
        if let Some(colon) = clean[bracket_end..].find(':') {
            let port_str = &clean[bracket_end + colon + 1..];
            let port = port_str.parse::<u16>().ok()?;
            let ip = clean[1..bracket_end].to_string();
            return Some((ip, port));
        }
    }

    if let Some(colon) = clean.rfind(':') {
        let ip_part = &clean[..colon];
        let port_part = &clean[colon + 1..];
        let port = port_part.parse::<u16>().ok()?;
        let ip = if ip_part == "*" {
            "0.0.0.0".to_string()
        } else {
            ip_part.to_string()
        };
        return Some((ip, port));
    }

    None
}

fn parse_ss_users(line: &str) -> (Option<u32>, String) {
    if let Some(users_idx) = line.find("users:((") {
        let slice = &line[users_idx + 8..];
        let mut name = String::from("unknown");
        let mut pid = None;

        if let Some(first_quote) = slice.find('"') {
            if let Some(second_quote) = slice[first_quote + 1..].find('"') {
                name = slice[first_quote + 1..first_quote + 1 + second_quote].to_string();
            }
        }

        if let Some(pid_idx) = slice.find("pid=") {
            let after_pid = &slice[pid_idx + 4..];
            let digits: String = after_pid
                .chars()
                .take_while(|c| c.is_ascii_digit())
                .collect();
            pid = digits.parse::<u32>().ok();
        }

        (pid, name)
    } else {
        (None, "unknown".to_string())
    }
}

fn resolve_proc_info(
    sys: &System,
    pid_opt: Option<u32>,
    fallback_name: &str,
) -> (String, f64, f32, String) {
    if let Some(pid) = pid_opt {
        if let Some(proc_entry) = sys.process(Pid::from(pid as usize)) {
            let name = proc_entry.name().to_string_lossy().to_string();
            let cmdline = proc_entry
                .cmd()
                .iter()
                .map(|s| s.to_string_lossy().to_string())
                .collect::<Vec<_>>()
                .join(" ");
            let mem_mb = proc_entry.memory() as f64 / (1024.0 * 1024.0);
            let cpu = proc_entry.cpu_usage();
            let effective_name = if !name.is_empty() {
                name
            } else {
                fallback_name.to_string()
            };
            return (cmdline, mem_mb, cpu, effective_name);
        }
    }

    ("".to_string(), 0.0, 0.0, fallback_name.to_string())
}

pub fn kill_process(pid: u32, force: bool) -> anyhow::Result<()> {
    let signal = if force { "-9" } else { "-15" };
    let status = Command::new("kill")
        .args([signal, &pid.to_string()])
        .status()?;

    if status.success() {
        Ok(())
    } else {
        anyhow::bail!(
            "Failed to terminate PID {} with exit status {:?}",
            pid,
            status
        )
    }
}

pub fn open_in_browser(url: &str) -> anyhow::Result<()> {
    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open").arg(url).spawn()?;
        Ok(())
    }
    #[cfg(target_os = "macos")]
    {
        Command::new("open").arg(url).spawn()?;
        Ok(())
    }
    #[cfg(target_os = "windows")]
    {
        Command::new("cmd").args(["/C", "start", url]).spawn()?;
        Ok(())
    }
}
