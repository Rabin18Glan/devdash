use serde::Deserialize;
use std::process::Command;

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct DockerContainer {
    #[serde(rename = "ID")]
    pub id: String,
    #[serde(rename = "Names")]
    pub names: String,
    #[serde(rename = "Image")]
    pub image: String,
    #[serde(rename = "Status")]
    pub status: String,
    #[serde(rename = "State")]
    pub state: String,
    #[serde(rename = "Ports")]
    pub ports: String,
    #[serde(rename = "CreatedAt")]
    pub created_at: String,
    #[serde(rename = "Command")]
    pub command: String,
}

impl DockerContainer {
    pub fn short_id(&self) -> &str {
        if self.id.len() >= 12 {
            &self.id[..12]
        } else {
            &self.id
        }
    }

    pub fn is_running(&self) -> bool {
        self.state.eq_ignore_ascii_case("running")
    }
}

pub fn check_docker_available() -> bool {
    Command::new("docker")
        .args(["info"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn list_containers() -> anyhow::Result<Vec<DockerContainer>> {
    let output = Command::new("docker")
        .args(["ps", "-a", "--format", "{{json .}}"])
        .output()?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Docker error: {}", err.trim());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut containers = Vec::new();

    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Ok(container) = serde_json::from_str::<DockerContainer>(line) {
            containers.push(container);
        }
    }

    Ok(containers)
}

pub fn start_container(id: &str) -> anyhow::Result<()> {
    let output = Command::new("docker").args(["start", id]).output()?;
    if output.status.success() {
        Ok(())
    } else {
        anyhow::bail!(
            "Failed to start: {}",
            String::from_utf8_lossy(&output.stderr)
        )
    }
}

pub fn stop_container(id: &str) -> anyhow::Result<()> {
    let output = Command::new("docker").args(["stop", id]).output()?;
    if output.status.success() {
        Ok(())
    } else {
        anyhow::bail!(
            "Failed to stop: {}",
            String::from_utf8_lossy(&output.stderr)
        )
    }
}

pub fn restart_container(id: &str) -> anyhow::Result<()> {
    let output = Command::new("docker").args(["restart", id]).output()?;
    if output.status.success() {
        Ok(())
    } else {
        anyhow::bail!(
            "Failed to restart: {}",
            String::from_utf8_lossy(&output.stderr)
        )
    }
}

pub fn get_container_logs(id: &str, tail: usize) -> anyhow::Result<String> {
    let tail_str = tail.to_string();
    let output = Command::new("docker")
        .args(["logs", "--tail", &tail_str, id])
        .output()?;

    let mut logs = String::from_utf8_lossy(&output.stdout).to_string();
    let err_logs = String::from_utf8_lossy(&output.stderr);
    if !err_logs.is_empty() {
        if !logs.is_empty() {
            logs.push('\n');
        }
        logs.push_str(&err_logs);
    }

    Ok(logs)
}
