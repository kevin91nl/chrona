use crate::db::JobRepository;
use crate::models::{DiscoveredJob, Job, JobStatus};
use crate::providers::SchedulerProvider;
use chrono::Utc;
use log::{debug, info, warn};
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
        discovered_at: Utc::now(),
        updated_at: Utc::now(),
        next_execution: None,
        previous_execution: None,
    }
}

fn sanitize_id(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
        .collect::<String>()
        .to_lowercase()
}