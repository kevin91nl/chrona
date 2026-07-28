use super::interface::{ProviderError, SchedulerProvider};
use crate::models::DiscoveredJob;
use std::process::Command;

pub struct LaunchdProvider;

impl LaunchdProvider {
    pub fn new() -> Self {
        Self
    }

    fn scan_plists(&self) -> Result<Vec<DiscoveredJob>, ProviderError> {
        let mut jobs = Vec::new();

        let home = std::env::var("HOME").unwrap_or_default();

        let dirs: Vec<String> = vec![
            "/Library/LaunchDaemons".into(),
            "/Library/LaunchAgents".into(),
            format!("{}/Library/LaunchAgents", home),
        ];

        for dir in &dirs {
            let entries = match std::fs::read_dir(dir) {
                Ok(e) => e,
                Err(_) => continue,
            };

            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(true, |ext| ext != "plist") {
                    continue;
                }

                if let Some(job) = parse_plist(&path) {
                    jobs.push(job);
                }
            }
        }

        // Also use launchctl list to find active jobs
        if let Ok(output) = Command::new("launchctl").arg("list").output() {
            if output.status.success() {
                let content = String::from_utf8_lossy(&output.stdout);
                for line in content.lines().skip(1) {
                    // Skip header
                    let parts: Vec<&str> = line.split('\t').collect();
                    if parts.len() >= 3 {
                        let label = parts.last().unwrap_or(&"").trim();
                        if !label.is_empty()
                            && label.starts_with("com.")
                            && !jobs.iter().any(|j| j.source.contains(label))
                        {
                            // Found a launchd job not from plist scan
                            let _pid = parts[0];
                            let _status = parts[1];
                            let enabled_str = parts.get(2).unwrap_or(&"");
                            let enabled = *enabled_str != "-";

                            jobs.push(DiscoveredJob {
                                name: label.to_string(),
                                provider: "launchd".into(),
                                source: format!("launchctl: {}", label),
                                schedule: "N/A (launchd interval)".into(),
                                timezone: None,
                                command: format!("launchd job: {}", label),
                                working_directory: None,
                                enabled,
                            });
                        }
                    }
                }
            }
        }

        Ok(jobs)
    }
}

impl SchedulerProvider for LaunchdProvider {
    fn id(&self) -> &str {
        "launchd"
    }

    fn name(&self) -> &str {
        "launchd"
    }

    fn is_available(&self) -> bool {
        // macOS only
        cfg!(target_os = "macos")
    }

    fn discover(&self) -> Result<Vec<DiscoveredJob>, ProviderError> {
        self.scan_plists()
    }

    fn refresh(&self) -> Result<Vec<DiscoveredJob>, ProviderError> {
        self.scan_plists()
    }
}

fn parse_plist(path: &std::path::Path) -> Option<DiscoveredJob> {
    // Read plist as XML and extract key fields
    let content = std::fs::read_to_string(path).ok()?;

    let label = extract_plist_string(&content, "Label").unwrap_or_else(|| {
        path.file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default()
    });

    let program = extract_plist_string(&content, "Program")
        .or_else(|| {
            extract_plist_array(&content, "ProgramArguments")
                .and_then(|args| args.first().cloned())
        })
        .unwrap_or_default();

    let start_interval = extract_plist_integer(&content, "StartInterval")
        .map(|secs| format!("every {}s", secs));

    let schedule = start_interval.unwrap_or_else(|| "N/A (launchd event)".into());

    // Check Disabled key
    let disabled = extract_plist_bool(&content, "Disabled").unwrap_or(false);

    // Derive name from label or program
    let name = if label.is_empty() {
        program
            .split('/')
            .last()
            .unwrap_or("unknown")
            .to_string()
    } else {
        label.clone()
    };

    let working_dir = extract_plist_string(&content, "WorkingDirectory");

    Some(DiscoveredJob {
        name,
        provider: "launchd".into(),
        source: path.to_string_lossy().to_string(),
        schedule,
        timezone: None,
        command: program,
        working_directory: working_dir,
        enabled: !disabled,
    })
}

fn extract_plist_string(xml: &str, key: &str) -> Option<String> {
    // Simple XML plist parser — looks for <key>KEY</key><string>VALUE</string>
    let key_tag = format!("<key>{}</key>", key);
    let idx = xml.find(&key_tag)?;
    let after = &xml[idx + key_tag.len()..];
    let string_start = after.find("<string>")?;
    let val_start = string_start + "<string>".len();
    let val_end = after[val_start..].find("</string>")?;
    Some(after[val_start..val_start + val_end].to_string())
}

fn extract_plist_integer(xml: &str, key: &str) -> Option<i64> {
    let key_tag = format!("<key>{}</key>", key);
    let idx = xml.find(&key_tag)?;
    let after = &xml[idx + key_tag.len()..];
    let tag_start = after.find("<integer>")?;
    let val_start = tag_start + "<integer>".len();
    let val_end = after[val_start..].find("</integer>")?;
    after[val_start..val_start + val_end].trim().parse().ok()
}

fn extract_plist_bool(xml: &str, key: &str) -> Option<bool> {
    let key_tag = format!("<key>{}</key>", key);
    let idx = xml.find(&key_tag)?;
    let after = &xml[idx + key_tag.len()..];
    if after.trim_start().starts_with("<true/>") {
        Some(true)
    } else if after.trim_start().starts_with("<false/>") {
        Some(false)
    } else {
        None
    }
}

fn extract_plist_array(xml: &str, key: &str) -> Option<Vec<String>> {
    let key_tag = format!("<key>{}</key>", key);
    let idx = xml.find(&key_tag)?;
    let after = &xml[idx + key_tag.len()..];
    let arr_start = after.find("<array>")?;
    let arr_content = &after[arr_start + "<array>".len()..];
    let arr_end = arr_content.find("</array>")?;
    let arr_str = &arr_content[..arr_end];

    let mut items = Vec::new();
    let mut remaining = arr_str;
    while let Some(s) = remaining.find("<string>") {
        let val_start = s + "<string>".len();
        if let Some(e) = remaining[val_start..].find("</string>") {
            items.push(remaining[val_start..val_start + e].to_string());
            remaining = &remaining[val_start + e + "</string>".len()..];
        } else {
            break;
        }
    }

    Some(items)
}