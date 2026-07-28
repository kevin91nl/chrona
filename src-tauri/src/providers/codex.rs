use super::interface::{ProviderError, SchedulerProvider};
use crate::models::DiscoveredJob;
use serde::Deserialize;
use std::path::PathBuf;

/// Codex Scheduled Tasks provider.
///
/// Scans `~/.codex/automations/*/automation.toml` for Codex automation configs.
/// Each automation has an RRULE-based schedule, model, and prompt.
pub struct CodexProvider;

#[derive(Debug, Deserialize)]
struct AutomationConfig {
    id: String,
    name: String,
    kind: String,
    status: String,
    rrule: Option<String>,
    model: Option<String>,
    prompt: Option<String>,
    cwds: Option<Vec<String>>,
    created_at: Option<i64>,
    updated_at: Option<i64>,
}

impl CodexProvider {
    pub fn new() -> Self {
        Self
    }

    fn automations_dir(&self) -> Option<PathBuf> {
        dirs_home().map(|h| PathBuf::from(h).join(".codex").join("automations"))
    }

    fn scan_automations(&self) -> Result<Vec<DiscoveredJob>, ProviderError> {
        let dir = match self.automations_dir() {
            Some(d) => d,
            None => return Ok(vec![]),
        };

        if !dir.exists() {
            return Ok(vec![]);
        }

        let mut jobs = Vec::new();

        let entries = std::fs::read_dir(&dir)?;
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let toml_path = path.join("automation.toml");
            if !toml_path.exists() {
                continue;
            }

            match std::fs::read_to_string(&toml_path) {
                Ok(content) => match toml::from_str::<AutomationConfig>(&content) {
                    Ok(config) => {
                        let schedule = config
                            .rrule
                            .as_deref()
                            .map(rrule_to_human)
                            .unwrap_or_else(|| "unknown schedule".to_string());

                        let working_dir = config
                            .cwds
                            .as_ref()
                            .and_then(|dirs| dirs.first().cloned());

                        let status_str = config.status.to_uppercase();
                        let enabled = status_str == "ACTIVE";

                        let model_info = config
                            .model
                            .as_deref()
                            .map(|m| format!(" [{}]", m))
                            .unwrap_or_default();

                        jobs.push(DiscoveredJob {
                            name: config.name.clone(),
                            provider: "codex".into(),
                            source: toml_path.to_string_lossy().to_string(),
                            schedule,
                            timezone: None,
                            command: format!("codex automation: {}{}", config.id, model_info),
                            working_directory: working_dir,
                            enabled,
                        });
                    }
                    Err(_) => {
                        // TOML parse error — skip silently
                    }
                },
                Err(_) => continue,
            }
        }

        Ok(jobs)
    }
}

impl SchedulerProvider for CodexProvider {
    fn id(&self) -> &str {
        "codex"
    }

    fn name(&self) -> &str {
        "Codex Scheduled Tasks"
    }

    fn is_available(&self) -> bool {
        self.automations_dir()
            .map(|d| d.exists())
            .unwrap_or(false)
    }

    fn discover(&self) -> Result<Vec<DiscoveredJob>, ProviderError> {
        self.scan_automations()
    }

    fn refresh(&self) -> Result<Vec<DiscoveredJob>, ProviderError> {
        self.scan_automations()
    }
}

fn dirs_home() -> Option<String> {
    std::env::var("HOME").ok()
}

/// Convert RRULE string to human-readable schedule.
/// Supports common Codex patterns: MINUTELY, HOURLY, DAILY, WEEKLY.
fn rrule_to_human(rrule: &str) -> String {
    let rrule = rrule.strip_prefix("RRULE:").unwrap_or(rrule);

    let freq = rrule
        .split(';')
        .find(|p| p.starts_with("FREQ="))
        .map(|p| &p[5..]);

    let interval = rrule
        .split(';')
        .find(|p| p.starts_with("INTERVAL="))
        .and_then(|p| p[9..].parse::<u32>().ok())
        .unwrap_or(1);

    let by_day = rrule
        .split(';')
        .find(|p| p.starts_with("BYDAY="))
        .map(|p| &p[6..]);

    match freq {
        Some("MINUTELY") => {
            if interval == 1 {
                "Every minute".to_string()
            } else {
                format!("Every {} minutes", interval)
            }
        }
        Some("HOURLY") => {
            if interval == 1 {
                "Every hour".to_string()
            } else {
                format!("Every {} hours", interval)
            }
        }
        Some("DAILY") => {
            if interval == 1 {
                "Daily".to_string()
            } else {
                format!("Every {} days", interval)
            }
        }
        Some("WEEKLY") => {
            let days = by_day
                .map(|d| format!(" on {}", d))
                .unwrap_or_default();
            if interval == 1 {
                format!("Weekly{}", days)
            } else {
                format!("Every {} weeks{}", interval, days)
            }
        }
        Some("MONTHLY") => {
            if interval == 1 {
                "Monthly".to_string()
            } else {
                format!("Every {} months", interval)
            }
        }
        _ => rrule.to_string(),
    }
}