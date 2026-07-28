use super::interface::{ProviderError, SchedulerProvider};
use crate::models::DiscoveredJob;

pub struct CronProvider;

impl CronProvider {
    pub fn new() -> Self {
        Self
    }

    fn scan_crontab(&self) -> Result<Vec<DiscoveredJob>, ProviderError> {
        let mut jobs = Vec::new();

        // User crontab
        if let Ok(output) = std::process::Command::new("crontab").arg("-l").output() {
            if output.status.success() {
                let content = String::from_utf8_lossy(&output.stdout);
                jobs.extend(parse_crontab(&content, "crontab (user)"));
            }
        }

        // /etc/crontab
        if let Ok(content) = std::fs::read_to_string("/etc/crontab") {
            jobs.extend(parse_crontab(&content, "/etc/crontab"));
        }

        // /etc/cron.d/
        if let Ok(entries) = std::fs::read_dir("/etc/cron.d") {
            for entry in entries.flatten() {
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    let source = entry.path().to_string_lossy().to_string();
                    jobs.extend(parse_crontab(&content, &source));
                }
            }
        }

        // /var/spool/cron/crontabs/
        if let Ok(entries) = std::fs::read_dir("/var/spool/cron/crontabs") {
            for entry in entries.flatten() {
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    let source = format!(
                        "/var/spool/cron/crontabs/{}",
                        entry.file_name().to_string_lossy()
                    );
                    jobs.extend(parse_crontab(&content, &source));
                }
            }
        }

        // macOS: /var/at/tabs/
        if let Ok(entries) = std::fs::read_dir("/var/at/tabs") {
            for entry in entries.flatten() {
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    let source = format!(
                        "/var/at/tabs/{}",
                        entry.file_name().to_string_lossy()
                    );
                    jobs.extend(parse_crontab(&content, &source));
                }
            }
        }

        Ok(jobs)
    }
}

impl SchedulerProvider for CronProvider {
    fn id(&self) -> &str {
        "cron"
    }

    fn name(&self) -> &str {
        "cron"
    }

    fn is_available(&self) -> bool {
        // Available if crontab exists on PATH
        std::process::Command::new("which")
            .arg("crontab")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    fn discover(&self) -> Result<Vec<DiscoveredJob>, ProviderError> {
        self.scan_crontab()
    }

    fn refresh(&self) -> Result<Vec<DiscoveredJob>, ProviderError> {
        self.scan_crontab()
    }
}

/// Parse crontab lines into DiscoveredJob entries.
/// Skips comments and empty lines.
fn parse_crontab(content: &str, source: &str) -> Vec<DiscoveredJob> {
    let mut jobs = Vec::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Skip environment variable assignments (key=value)
        if !line.contains('\t') && !line.contains("  ") && line.contains('=') && !line.starts_with('*') && !line.starts_with('@') {
            let before_eq = line.split('=').next().unwrap_or("");
            if before_eq.chars().all(|c| c.is_alphanumeric() || c == '_') {
                continue;
            }
        }

        if let Some(job) = parse_cron_line(line, source) {
            jobs.push(job);
        }
    }

    jobs
}

fn parse_cron_line(line: &str, source: &str) -> Option<DiscoveredJob> {
    // Handle @reboot, @hourly, @daily, etc.
    if line.starts_with('@') {
        let parts: Vec<&str> = line.splitn(2, char::is_whitespace).collect();
        if parts.len() < 2 {
            return None;
        }
        let schedule = parts[0].to_string();
        let command = parts[1].trim().to_string();
        let name = command_name(&command);

        return Some(DiscoveredJob {
            name,
            provider: "cron".into(),
            source: source.into(),
            schedule,
            timezone: None,
            command,
            working_directory: None,
            enabled: true,
        });
    }

    // Standard: min hour dom month dow [user] command
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 6 {
        return None;
    }

    // Check if first 5 fields look like cron schedule
    let schedule = format!("{} {} {} {} {}", parts[0], parts[1], parts[2], parts[3], parts[4]);

    // Command is everything after the schedule (and optional user field)
    // In /etc/crontab and /etc/cron.d, there's a user field
    // In user crontabs, command starts at position 5
    // We detect by source path
    let (command_start_idx, _user) = if source.contains("/etc/")
        || source.contains("/var/spool/")
    {
        // System crontab: field 6+ is the command, field 5 is user
        (6, Some(parts[5].to_string()))
    } else if source == "crontab (user)" {
        // User crontab: field 5+ is the command
        (5, None)
    } else {
        (5, None)
    };

    if parts.len() <= command_start_idx {
        return None;
    }

    let command = parts[command_start_idx..].join(" ");
    if command.is_empty() {
        return None;
    }

    let name = command_name(&command);

    Some(DiscoveredJob {
        name,
        provider: "cron".into(),
        source: source.into(),
        schedule,
        timezone: None,
        command,
        working_directory: None,
        enabled: true,
    })
}

fn command_name(command: &str) -> String {
    // Extract a reasonable job name from a command
    let path = command.split_whitespace().next().unwrap_or(command);
    std::path::Path::new(path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| {
            // Truncate long commands
            if command.len() > 50 {
                format!("{}...", &command[..47])
            } else {
                command.to_string()
            }
        })
}
