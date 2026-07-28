use super::interface::{ProviderError, SchedulerProvider};
use crate::models::DiscoveredJob;
use std::process::Command;

/// Systemd timer provider — scans systemd timers.
/// Supported platforms: Linux
#[allow(dead_code)]
pub struct SystemdProvider;

impl SystemdProvider {
    pub fn new() -> Self {
        Self
    }

    fn scan_timers(&self) -> Result<Vec<DiscoveredJob>, ProviderError> {
        let mut jobs = Vec::new();

        // List all timer units
        let output = Command::new("systemctl")
            .args(["list-timers", "--all", "--no-pager", "--output", "json"])
            .output()?;

        if !output.status.success() {
            // Fallback: list-timers without JSON
            return self.scan_timers_fallback();
        }

        let content = String::from_utf8_lossy(&output.stdout);

        // Parse each JSON line
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('[') || line.starts_with(']') {
                continue;
            }

            // Try parsing as JSON object
            if let Ok(timer) = serde_json::from_str::<serde_json::Value>(line) {
                let unit = timer
                    .get("unit")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");

                let schedule = timer
                    .get("next")
                    .and_then(|v| v.as_str())
                    .unwrap_or("N/A")
                    .to_string();

                let command = timer
                    .get("activates")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let name = unit.replace(".timer", "");

                // Get the underlying service unit file path
                let source = format!("systemd: {}", unit);

                jobs.push(DiscoveredJob {
                    name,
                    provider: "systemd".into(),
                    source,
                    schedule,
                    timezone: None,
                    command: if command.is_empty() {
                        format!("activates: {}", unit)
                    } else {
                        command
                    },
                    working_directory: None,
                    enabled: true, // If it shows in list-timers, it's enabled
                });
            }
        }

        Ok(jobs)
    }

    fn scan_timers_fallback(&self) -> Result<Vec<DiscoveredJob>, ProviderError> {
        let mut jobs = Vec::new();

        let output = Command::new("systemctl")
            .args(["list-timers", "--all", "--no-pager"])
            .output()?;

        if !output.status.success() {
            return Ok(jobs);
        }

        let content = String::from_utf8_lossy(&output.stdout);

        for line in content.lines().skip(1) {
            // Skip header
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 5 {
                continue;
            }

            // LAST column might be empty or have a value
            // NEXT LEFT LAST LEFT UNIT ACTIVATES
            let next_col = parts[0];
            let unit = parts.iter().find(|p| p.ends_with(".timer"));

            if let Some(timer_unit) = unit {
                let name = timer_unit.replace(".timer", "");
                let remaining = line
                    .find(timer_unit)
                    .map(|i| &line[i + timer_unit.len()..])
                    .unwrap_or("")
                    .trim();

                jobs.push(DiscoveredJob {
                    name,
                    provider: "systemd".into(),
                    source: format!("systemd: {}", timer_unit),
                    schedule: format!("Next: {}", next_col),
                    timezone: None,
                    command: remaining.to_string(),
                    working_directory: None,
                    enabled: true,
                });
            }
        }

        Ok(jobs)
    }
}

impl SchedulerProvider for SystemdProvider {
    fn id(&self) -> &str {
        "systemd"
    }

    fn name(&self) -> &str {
        "systemd timers"
    }

    fn is_available(&self) -> bool {
        // Linux only — check if systemd is present
        cfg!(target_os = "linux")
            && Command::new("systemctl")
                .arg("--version")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
    }

    fn discover(&self) -> Result<Vec<DiscoveredJob>, ProviderError> {
        self.scan_timers()
    }

    fn refresh(&self) -> Result<Vec<DiscoveredJob>, ProviderError> {
        self.scan_timers()
    }
}