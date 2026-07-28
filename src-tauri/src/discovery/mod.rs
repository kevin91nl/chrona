use crate::db::JobRepository;
use crate::models::{DiscoveredJob, Job, JobStatus};
use crate::providers::SchedulerProvider;
use chrono::Utc;
use log::{debug, info, warn};
use std::str::FromStr;
use std::sync::Arc;

/// Discovery engine that orchestrates provider scanning
/// and normalizes results into the local database.
pub struct DiscoveryEngine {
    repo: Arc<JobRepository>,
    providers: Vec<Box<dyn SchedulerProvider>>,
}

/// Info about a detected scheduler.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SchedulerInfo {
    pub id: String,
    pub name: String,
    pub available: bool,
    pub job_count: usize,
}

impl DiscoveryEngine {
    pub fn new(repo: Arc<JobRepository>, providers: Vec<Box<dyn SchedulerProvider>>) -> Self {
        Self { repo, providers }
    }

    /// Run full discovery across all available providers.
    pub fn discover_all(&self) -> Result<Vec<Job>, String> {
        let mut all_jobs = Vec::new();

        for provider in &self.providers {
            if !provider.is_available() {
                debug!("Skipping unavailable provider: {}", provider.name());
                continue;
            }

            info!("Running discovery for provider: {}", provider.name());

            match provider.discover() {
                Ok(discovered) => {
                    info!(
                        "Provider {} discovered {} jobs",
                        provider.name(),
                        discovered.len()
                    );

                    for dj in &discovered {
                        let job = normalize_job(dj, provider.id());
                        if let Err(e) = self.repo.upsert_job(&job) {
                            warn!("Failed to upsert job {}: {}", job.id, e);
                        } else {
                            all_jobs.push(job);
                        }
                    }

                    // Remove stale jobs from this provider
                    let active_ids: Vec<String> = all_jobs
                        .iter()
                        .filter(|j| j.provider == provider.id())
                        .map(|j| j.id.clone())
                        .collect();

                    if let Err(e) = self.repo.remove_stale_jobs(provider.id(), &active_ids) {
                        warn!("Failed to remove stale jobs for {}: {}", provider.name(), e);
                    }
                }
                Err(e) => {
                    warn!("Discovery failed for {}: {}", provider.name(), e);
                }
            }
        }

        Ok(all_jobs)
    }

    /// Refresh — detect new, changed, removed jobs incrementally.
    pub fn refresh(&self) -> Result<Vec<Job>, String> {
        self.discover_all()
    }

    /// Get scheduler info for all registered providers.
    pub fn get_scheduler_info(&self) -> Vec<SchedulerInfo> {
        let counts = self
            .repo
            .get_job_count_by_provider()
            .unwrap_or_default();

        let count_map: std::collections::HashMap<String, i64> = counts.into_iter().collect();

        self.providers
            .iter()
            .map(|p| {
                let id = p.id().to_string();
                SchedulerInfo {
                    id: id.clone(),
                    name: p.name().to_string(),
                    available: p.is_available(),
                    job_count: count_map.get(&id).copied().unwrap_or(0) as usize,
                }
            })
            .collect()
    }
}

fn normalize_job(discovered: &DiscoveredJob, provider_id: &str) -> Job {
    let id = format!(
        "{}-{}",
        provider_id,
        sanitize_id(&discovered.name)
    );

    let now = Utc::now();
    let next_execution = compute_next_execution(&discovered.schedule, provider_id, now);

    Job {
        id,
        name: discovered.name.clone(),
        provider: discovered.provider.clone(),
        source: discovered.source.clone(),
        schedule: discovered.schedule.clone(),
        timezone: discovered.timezone.clone(),
        command: discovered.command.clone(),
        working_directory: discovered.working_directory.clone(),
        status: if discovered.enabled {
            JobStatus::Active
        } else {
            JobStatus::Inactive
        },
        enabled: discovered.enabled,
        discovered_at: now,
        updated_at: now,
        next_execution,
        previous_execution: None,
    }
}

fn sanitize_id(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
        .collect::<String>()
        .to_lowercase()
}

/// Compute the next execution time from a schedule string.
/// Provider-aware: cron uses `cron` crate for standard expressions,
/// launchd parses interval strings, codex returns None (RRULE not supported).
fn compute_next_execution(
    schedule: &str,
    provider: &str,
    now: chrono::DateTime<Utc>,
) -> Option<chrono::DateTime<Utc>> {
    let s = schedule.trim();

    match provider {
        "cron" => compute_cron_next(s, now),
        "launchd" => compute_launchd_next(s, now),
        "codex" => compute_codex_next(s, now),
        _ => compute_cron_next(s, now).or_else(|| compute_codex_next(s, now)),
    }
}

/// Standard cron expressions via the `cron` crate.
/// The crate expects 7-field format: sec min hour dom month dow year.
fn compute_cron_next(schedule: &str, now: chrono::DateTime<Utc>) -> Option<chrono::DateTime<Utc>> {
    let five_field = match schedule {
        "@hourly" => "0 * * * *",
        "@daily" | "@midnight" => "0 0 * * *",
        "@weekly" => "0 0 * * 0",
        "@monthly" => "0 0 1 * *",
        "@reboot" => return None,
        other => {
            let parts: Vec<&str> = other.split_whitespace().collect();
            if parts.len() != 5 {
                return None;
            }
            other
        }
    };
    // Convert 5-field to 7-field: prepend sec=0, append year=*
    let expr = format!("0 {} *", five_field);
    let schedule = cron::Schedule::from_str(&expr).ok()?;
    schedule.after(&now).next()
}

/// launchd interval: "Every N seconds" or "Every N minutes" or "Every N hours".
fn compute_launchd_next(schedule: &str, now: chrono::DateTime<Utc>) -> Option<chrono::DateTime<Utc>> {
    let lower = schedule.to_lowercase();
    let parts: Vec<&str> = lower.split_whitespace().collect();
    if parts.len() < 3 || parts[0] != "every" {
        return None;
    }
    let value: i64 = parts[1].parse().ok()?;
    if value <= 0 {
        return None;
    }
    match parts[2] {
        "second" | "seconds" => Some(now + chrono::Duration::seconds(value)),
        "minute" | "minutes" => Some(now + chrono::Duration::minutes(value)),
        "hour" | "hours" => Some(now + chrono::Duration::hours(value)),
        "day" | "days" => Some(now + chrono::Duration::days(value)),
        _ => None,
    }
}

/// Codex RRULE: "Every N minutes/hours" (human-readable) or RRULE prefix.
/// Returns None for complex RRULE — only handles simple interval patterns.
fn compute_codex_next(schedule: &str, now: chrono::DateTime<Utc>) -> Option<chrono::DateTime<Utc>> {
    // Try human-readable first: "Every 15 minutes"
    if let Some(next) = compute_launchd_next(schedule, now) {
        return Some(next);
    }
    // RRULE not supported — caller should handle
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Datelike, TimeZone, Timelike};

    fn fixed_now() -> chrono::DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 7, 28, 14, 30, 0).unwrap()
    }

    // --- Bug 1: next_execution computed from schedule ---

    #[test]
    fn cron_standard_five_field() {
        let now = fixed_now();
        // "0 15 * * *" = at 15:00 daily → next is today 15:00 (14:30 < 15:00)
        let next = compute_next_execution("0 15 * * *", "cron", now).unwrap();
        assert_eq!(next.hour(), 15);
        assert_eq!(next.minute(), 0);
        assert_eq!(next.date_naive(), now.date_naive());
    }

    #[test]
    fn cron_standard_past_time_returns_tomorrow() {
        let now = fixed_now();
        // "0 2 * * *" = at 02:00 daily → next is tomorrow 02:00
        let next = compute_next_execution("0 2 * * *", "cron", now).unwrap();
        assert_eq!(next.hour(), 2);
        assert_eq!(next.date_naive(), now.date_naive() + chrono::Duration::days(1));
    }

    #[test]
    fn cron_step_every_fifteen_minutes() {
        let now = fixed_now();
        // "*/15 * * * *" = every 15 min → next is 14:45
        let next = compute_next_execution("*/15 * * * *", "cron", now).unwrap();
        assert_eq!(next.hour(), 14);
        assert_eq!(next.minute(), 45);
    }

    #[test]
    fn cron_at_hourly_keyword() {
        let now = fixed_now();
        let next = compute_next_execution("@hourly", "cron", now).unwrap();
        assert_eq!(next.hour(), 15);
        assert_eq!(next.minute(), 0);
    }

    #[test]
    fn cron_at_daily_keyword() {
        let now = fixed_now();
        let next = compute_next_execution("@daily", "cron", now).unwrap();
        assert_eq!(next.hour(), 0);
        assert_eq!(next.minute(), 0);
        assert_eq!(next.date_naive(), now.date_naive() + chrono::Duration::days(1));
    }

    #[test]
    fn cron_invalid_returns_none() {
        assert!(compute_next_execution("not-a-cron", "cron", fixed_now()).is_none());
        assert!(compute_next_execution("@reboot", "cron", fixed_now()).is_none());
    }

    // --- launchd intervals ---

    #[test]
    fn launchd_every_n_minutes() {
        let now = fixed_now();
        let next = compute_next_execution("Every 30 minutes", "launchd", now).unwrap();
        assert_eq!(next, now + chrono::Duration::minutes(30));
    }

    #[test]
    fn launchd_every_n_hours() {
        let now = fixed_now();
        let next = compute_next_execution("Every 6 hours", "launchd", now).unwrap();
        assert_eq!(next, now + chrono::Duration::hours(6));
    }

    // --- codex human-readable ---

    #[test]
    fn codex_every_fifteen_minutes() {
        let now = fixed_now();
        let next = compute_next_execution("Every 15 minutes", "codex", now).unwrap();
        assert_eq!(next, now + chrono::Duration::minutes(15));
    }

    #[test]
    fn codex_rrule_returns_none() {
        assert!(compute_next_execution("RRULE:FREQ=DAILY", "codex", fixed_now()).is_none());
    }

    // --- normalize_job produces next_execution ---

    #[test]
    fn normalize_job_has_next_execution() {
        let dj = DiscoveredJob {
            name: "test".into(),
            provider: "cron".into(),
            source: "/tmp/test".into(),
            schedule: "0 15 * * *".into(),
            timezone: None,
            command: "echo hi".into(),
            working_directory: None,
            enabled: true,
        };
        let job = normalize_job(&dj, "cron");
        assert!(job.next_execution.is_some(), "next_execution should be set");
    }

    #[test]
    fn normalize_job_unknown_schedule_returns_none() {
        let dj = DiscoveredJob {
            name: "test".into(),
            provider: "cron".into(),
            source: "/tmp/test".into(),
            schedule: "invalid".into(),
            timezone: None,
            command: "echo hi".into(),
            working_directory: None,
            enabled: true,
        };
        let job = normalize_job(&dj, "cron");
        assert!(job.next_execution.is_none());
    }

    #[test]
    fn cron_with_day_of_week() {
        let now = fixed_now(); // 2026-07-28 14:30 (Tuesday)
        // "0 9 * * 1" = at 09:00 on day-of-week 1
        let next = compute_next_execution("0 9 * * 1", "cron", now).unwrap();
        assert_eq!(next.hour(), 9);
        assert_eq!(next.minute(), 0);
        assert!(next > now, "next execution must be in the future");
        // Must be at least 1 day away (09:00 today already passed)
        assert!(next.signed_duration_since(now).num_hours() >= 12);
    }

    #[test]
    fn cron_with_specific_month() {
        let now = fixed_now(); // July 28
        // "0 12 15 8 *" = 15th of August at 12:00
        let next = compute_next_execution("0 12 15 8 *", "cron", now).unwrap();
        assert_eq!(next.month(), 8);
        assert_eq!(next.day(), 15);
        assert_eq!(next.hour(), 12);
        assert!(next > now);
    }

    #[test]
    fn cron_with_range_field() {
        let now = fixed_now();
        // "0 9-17 * * *" = every hour from 9 to 17 → next is 15:00
        let next = compute_next_execution("0 9-17 * * *", "cron", now).unwrap();
        assert_eq!(next.hour(), 15);
        assert_eq!(next.minute(), 0);
    }

    #[test]
    fn normalize_job_codex_uses_rrule_schedule() {
        let dj = DiscoveredJob {
            name: "test".into(),
            provider: "codex".into(),
            source: "/tmp/test".into(),
            schedule: "Every 15 minutes".into(),
            timezone: None,
            command: "echo hi".into(),
            working_directory: None,
            enabled: true,
        };
        let job = normalize_job(&dj, "codex");
        assert!(job.next_execution.is_some());
    }
}