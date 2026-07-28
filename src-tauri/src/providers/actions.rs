/// Provider-specific pause/resume actions.
///
/// These modify the actual source files so the OS scheduler respects the change.
use crate::models::Job;
use std::path::Path;

/// Toggle a job's enabled state at the provider level.
/// Returns the new enabled state.
pub fn toggle_provider_job(job: &Job) -> Result<bool, String> {
    match job.provider.as_str() {
        "codex" => toggle_codex(job),
        "cron" => toggle_cron(job),
        _ => Err(format!(
            "Provider '{}' does not support pause/resume",
            job.provider
        )),
    }
}

// ── Codex ────────────────────────────────────────────────────────

fn toggle_codex(job: &Job) -> Result<bool, String> {
    let path = Path::new(&job.source);
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {}", job.source, e))?;

    let new_enabled = !job.enabled;
    let new_status = if new_enabled { "ACTIVE" } else { "PAUSED" };

    // Replace the status line in the TOML.
    // Handles: status = "ACTIVE", status="ACTIVE", status = "active", etc.
    let new_content = replace_toml_status(&content, new_status);

    std::fs::write(path, new_content)
        .map_err(|e| format!("Failed to write {}: {}", job.source, e))?;

    Ok(new_enabled)
}

fn replace_toml_status(content: &str, new_value: &str) -> String {
    content
        .lines()
        .map(|line| {
            let trimmed = line.trim_start();
            if trimmed.starts_with("status") {
                // Replace the value after status = "..."
                // Preserve leading whitespace
                let indent: String = line.chars().take_while(|c| c.is_whitespace()).collect();
                format!("{}status = \"{}\"", indent, new_value)
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

// ── Cron ─────────────────────────────────────────────────────────

fn toggle_cron(job: &Job) -> Result<bool, String> {
    let source = &job.source;

    if source == "crontab (user)" {
        toggle_cron_user(job)
    } else {
        toggle_cron_file(job)
    }
}

/// User crontab via `crontab -l` / `crontab -`.
fn toggle_cron_user(job: &Job) -> Result<bool, String> {
    let output = std::process::Command::new("crontab")
        .arg("-l")
        .output()
        .map_err(|e| format!("Failed to run crontab -l: {}", e))?;

    if !output.status.success() {
        return Err("No user crontab found".to_string());
    }

    let content = String::from_utf8_lossy(&output.stdout);
    let new_content = toggle_cron_line(&content, &job.command, &job.schedule);
    let new_enabled = !job.enabled;

    // Write back via stdin to `crontab -`
    let mut child = std::process::Command::new("crontab")
        .arg("-")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn crontab: {}", e))?;

    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write;
        stdin
            .write_all(new_content.as_bytes())
            .map_err(|e| format!("Failed to write crontab stdin: {}", e))?;
    }

    let status = child
        .wait()
        .map_err(|e| format!("Failed to wait for crontab: {}", e))?;

    if !status.success() {
        let stderr = child
            .wait_with_output()
            .map(|o| String::from_utf8_lossy(&o.stderr).to_string())
            .unwrap_or_default();
        return Err(format!("crontab write failed: {}", stderr));
    }

    Ok(new_enabled)
}

/// File-based crontab (e.g. /etc/cron.d/*, /etc/crontab).
fn toggle_cron_file(job: &Job) -> Result<bool, String> {
    let path = Path::new(&job.source);
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {}", job.source, e))?;

    let new_content = toggle_cron_line(&content, &job.command, &job.schedule);
    let new_enabled = !job.enabled;

    std::fs::write(path, new_content)
        .map_err(|e| format!("Failed to write {}: {}", job.source, e))?;

    Ok(new_enabled)
}

/// Toggle comment on the crontab line matching command + schedule.
/// Comment = pause, uncomment = resume.
fn toggle_cron_line(content: &str, command: &str, schedule: &str) -> String {
    let command_trimmed = command.trim();
    let schedule_trimmed = schedule.trim();

    content
        .lines()
        .map(|line| {
            let trimmed = line.trim_start();

            // Check if this line contains both the schedule and command
            if !trimmed.starts_with('#') && trimmed.contains(schedule_trimmed) && trimmed.contains(command_trimmed) {
                // Comment out (pause)
                format!("# {}", line)
            } else if trimmed.starts_with("# ") {
                // Check if uncommented version would match
                let uncommented = &trimmed[2..];
                if uncommented.contains(schedule_trimmed) && uncommented.contains(command_trimmed) {
                    // Uncomment (resume) — preserve original indentation
                    let indent: String = line.chars().take_while(|c| c.is_whitespace()).collect();
                    format!("{}{}", indent, uncommented)
                } else {
                    line.to_string()
                }
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codex_replace_status_active_to_paused() {
        let input = "id = \"test\"\nname = \"Test\"\nstatus = \"ACTIVE\"\nrrule = \"...\"";
        let result = replace_toml_status(input, "PAUSED");
        assert!(result.contains("status = \"PAUSED\""));
        assert!(!result.contains("ACTIVE"));
    }

    #[test]
    fn codex_replace_status_paused_to_active() {
        let input = "status = \"PAUSED\"\nname = \"Test\"";
        let result = replace_toml_status(input, "ACTIVE");
        assert!(result.contains("status = \"ACTIVE\""));
    }

    #[test]
    fn codex_preserves_indentation() {
        let input = "  status = \"ACTIVE\"";
        let result = replace_toml_status(input, "PAUSED");
        assert_eq!(result, "  status = \"PAUSED\"");
    }

    #[test]
    fn cron_comment_out_matching_line() {
        let content = "0 2 * * * /usr/local/bin/backup.sh\n0 * * * * /opt/scripts/db-health.sh";
        let result = toggle_cron_line(content, "/usr/local/bin/backup.sh", "0 2 * * *");
        let lines: Vec<&str> = result.lines().collect();
        assert!(lines[0].starts_with("# "));
        assert!(!lines[1].starts_with('#'));
    }

    #[test]
    fn cron_uncomment_matching_line() {
        let content = "# 0 2 * * * /usr/local/bin/backup.sh\n0 * * * * /opt/scripts/db-health.sh";
        let result = toggle_cron_line(content, "/usr/local/bin/backup.sh", "0 2 * * *");
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines[0], "0 2 * * * /usr/local/bin/backup.sh");
    }

    #[test]
    fn cron_only_affects_matching_line() {
        let content = "0 2 * * * /usr/local/bin/backup.sh\n0 * * * * /opt/scripts/db-health.sh";
        let result = toggle_cron_line(content, "/opt/scripts/db-health.sh", "0 * * * *");
        let lines: Vec<&str> = result.lines().collect();
        assert!(!lines[0].starts_with('#'));
        assert!(lines[1].starts_with("# "));
    }
}
