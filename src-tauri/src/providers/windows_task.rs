use super::interface::{ProviderError, SchedulerProvider};
use crate::models::DiscoveredJob;
use std::process::Command;

/// Windows Task Scheduler provider.
/// Uses `schtasks` CLI on Windows.
#[allow(dead_code)]
pub struct WindowsTaskProvider;

impl WindowsTaskProvider {
    pub fn new() -> Self {
        Self
    }

    fn scan_tasks(&self) -> Result<Vec<DiscoveredJob>, ProviderError> {
        let output = Command::new("schtasks")
            .args(["/query", "/fo", "CSV", "/v", "/nh"])
            .output()?;

        if !output.status.success() {
            return Err(ProviderError::Other(
                "schtasks query failed".to_string(),
            ));
        }

        let content = String::from_utf8_lossy(&output.stdout);
        let mut jobs = Vec::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            // CSV fields: "TaskName","Next Run Time","Status","Logon Mode","Last Run Time","Last Result","Creator","Schedule","Schedule Type","Start Time","End Time","Days","Months","Repeat: Every","Repeat: Until: Time","Repeat: Until: Duration","Repeat: Stop If Still Running"
            let fields = parse_csv_line(line);

            if fields.len() < 8 {
                continue;
            }

            let task_name = fields[0].trim_matches('"');
            let status = fields[2].trim_matches('"');
            let schedule = fields[7].trim_matches('"');
            let schedule_type = fields.get(8).map(|s| s.trim_matches('"')).unwrap_or("");

            let enabled = status != "Disabled";
            let name = task_name
                .split('\\')
                .last()
                .unwrap_or(task_name)
                .to_string();

            jobs.push(DiscoveredJob {
                name,
                provider: "windows-task".into(),
                source: task_name.to_string(),
                schedule: format_schedule(schedule_type, schedule),
                timezone: None,
                command: format!("Windows scheduled task: {}", task_name),
                working_directory: None,
                enabled,
            });
        }

        Ok(jobs)
    }
}

impl SchedulerProvider for WindowsTaskProvider {
    fn id(&self) -> &str {
        "windows-task"
    }

    fn name(&self) -> &str {
        "Windows Task Scheduler"
    }

    fn is_available(&self) -> bool {
        cfg!(target_os = "windows")
    }

    fn discover(&self) -> Result<Vec<DiscoveredJob>, ProviderError> {
        self.scan_tasks()
    }

    fn refresh(&self) -> Result<Vec<DiscoveredJob>, ProviderError> {
        self.scan_tasks()
    }
}

fn parse_csv_line(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;

    for ch in line.chars() {
        match ch {
            '"' => in_quotes = !in_quotes,
            ',' if !in_quotes => {
                fields.push(current.clone());
                current.clear();
            }
            _ => current.push(ch),
        }
    }
    fields.push(current);
    fields
}

fn format_schedule(schedule_type: &str, schedule: &str) -> String {
    if schedule_type.is_empty() || schedule_type == "N/A" {
        return schedule.to_string();
    }
    format!("{}: {}", schedule_type, schedule)
}