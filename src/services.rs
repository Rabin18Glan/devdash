use std::process::Command;

#[derive(Clone, Debug, PartialEq)]
pub struct SystemdService {
    pub unit: String,
    pub load: String,
    pub active: String, // "active", "inactive", "failed"
    pub sub: String,    // "running", "dead", "failed", "exited"
    pub description: String,
    pub is_user: bool,
}

impl SystemdService {
    pub fn is_running(&self) -> bool {
        self.active.eq_ignore_ascii_case("active") && self.sub.eq_ignore_ascii_case("running")
    }

    pub fn is_failed(&self) -> bool {
        self.active.eq_ignore_ascii_case("failed") || self.sub.eq_ignore_ascii_case("failed")
    }
}

pub fn list_services(user_mode: bool) -> anyhow::Result<Vec<SystemdService>> {
    let mut cmd = Command::new("systemctl");
    if user_mode {
        cmd.arg("--user");
    }
    cmd.args([
        "list-units",
        "--type=service",
        "--all",
        "--no-legend",
        "--no-pager",
    ]);

    let output = cmd.output()?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("systemctl failed: {}", err.trim());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut services = Vec::new();

    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Lines can start with "● " if failed or bullet point
        let clean_line = trimmed
            .trim_start_matches('●')
            .trim_start_matches('*')
            .trim();
        let parts: Vec<&str> = clean_line.split_whitespace().collect();
        if parts.len() < 4 {
            continue;
        }

        let unit = parts[0].to_string();
        let load = parts[1].to_string();
        let active = parts[2].to_string();
        let sub = parts[3].to_string();
        let description = if parts.len() > 4 {
            parts[4..].join(" ")
        } else {
            String::new()
        };

        services.push(SystemdService {
            unit,
            load,
            active,
            sub,
            description,
            is_user: user_mode,
        });
    }

    // Sort active and running services near the top, then alphabetically
    services.sort_by(|a, b| {
        let a_score = if a.is_running() {
            0
        } else if a.is_failed() {
            1
        } else {
            2
        };
        let b_score = if b.is_running() {
            0
        } else if b.is_failed() {
            1
        } else {
            2
        };
        a_score.cmp(&b_score).then_with(|| a.unit.cmp(&b.unit))
    });

    Ok(services)
}

pub fn start_service(unit: &str, user_mode: bool) -> anyhow::Result<()> {
    let mut cmd = Command::new("systemctl");
    if user_mode {
        cmd.arg("--user");
    }
    cmd.args(["start", unit]);
    let status = cmd.status()?;
    if status.success() {
        Ok(())
    } else {
        anyhow::bail!("Failed to start service {}", unit)
    }
}

pub fn stop_service(unit: &str, user_mode: bool) -> anyhow::Result<()> {
    let mut cmd = Command::new("systemctl");
    if user_mode {
        cmd.arg("--user");
    }
    cmd.args(["stop", unit]);
    let status = cmd.status()?;
    if status.success() {
        Ok(())
    } else {
        anyhow::bail!("Failed to stop service {}", unit)
    }
}

pub fn restart_service(unit: &str, user_mode: bool) -> anyhow::Result<()> {
    let mut cmd = Command::new("systemctl");
    if user_mode {
        cmd.arg("--user");
    }
    cmd.args(["restart", unit]);
    let status = cmd.status()?;
    if status.success() {
        Ok(())
    } else {
        anyhow::bail!("Failed to restart service {}", unit)
    }
}

pub fn disable_now(unit: &str, user_mode: bool) -> anyhow::Result<()> {
    let mut cmd = Command::new("systemctl");
    if user_mode {
        cmd.arg("--user");
    }
    cmd.args(["disable", "--now", unit]);
    let output = cmd.output()?;
    if output.status.success() {
        Ok(())
    } else {
        let err = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to disable service: {}", err.trim())
    }
}

pub fn enable_now(unit: &str, user_mode: bool) -> anyhow::Result<()> {
    let mut cmd = Command::new("systemctl");
    if user_mode {
        cmd.arg("--user");
    }
    cmd.args(["enable", "--now", unit]);
    let output = cmd.output()?;
    if output.status.success() {
        Ok(())
    } else {
        let err = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to enable service: {}", err.trim())
    }
}

pub fn get_service_logs(unit: &str, user_mode: bool, lines: usize) -> anyhow::Result<String> {
    let mut cmd = Command::new("journalctl");
    if user_mode {
        cmd.arg("--user");
    }
    let n_str = lines.to_string();
    cmd.args(["-u", unit, "-n", &n_str, "--no-pager"]);

    let output = cmd.output()?;
    let logs = String::from_utf8_lossy(&output.stdout).to_string();
    if logs.trim().is_empty() {
        let err = String::from_utf8_lossy(&output.stderr);
        if !err.trim().is_empty() {
            return Ok(err.to_string());
        }
        return Ok("(No logs found in journal)".to_string());
    }
    Ok(logs)
}

pub fn get_service_status_text(unit: &str, user_mode: bool) -> anyhow::Result<String> {
    let mut cmd = Command::new("systemctl");
    if user_mode {
        cmd.arg("--user");
    }
    cmd.args(["status", unit, "--no-pager"]);
    let output = cmd.output()?;
    let text = String::from_utf8_lossy(&output.stdout).to_string();
    if text.is_empty() {
        Ok(String::from_utf8_lossy(&output.stderr).to_string())
    } else {
        Ok(text)
    }
}
