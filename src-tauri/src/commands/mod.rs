use crate::discovery::SchedulerInfo;
use crate::models::{Execution, Job};
use crate::AppState;
use tauri::State;

#[tauri::command]
pub fn get_jobs(state: State<AppState>) -> Result<Vec<Job>, String> {
    state.repo.get_all_jobs().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_job(state: State<AppState>, id: String) -> Result<Option<Job>, String> {
    state.repo.get_job(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_executions(state: State<AppState>, job_id: String) -> Result<Vec<Execution>, String> {
    state
        .repo
        .get_executions(&job_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_schedulers(state: State<AppState>) -> Result<Vec<SchedulerInfo>, String> {
    Ok(state.discovery.get_scheduler_info())
}

#[tauri::command]
pub fn get_discovery_stats(state: State<AppState>) -> Result<DiscoveryStats, String> {
    let jobs = state.repo.get_all_jobs().map_err(|e| e.to_string())?;
    let schedulers = state.discovery.get_scheduler_info();

    let total = jobs.len();
    let active = jobs
        .iter()
        .filter(|j| j.status == crate::models::JobStatus::Active)
        .count();
    let failed = jobs
        .iter()
        .filter(|j| j.status == crate::models::JobStatus::Error)
        .count();
    let schedulers_detected = schedulers.iter().filter(|s| s.available).count();

    let next = jobs
        .iter()
        .filter_map(|j| {
            j.next_execution.map(|t| {
                let time_str = t.format("%Y-%m-%d %H:%M:%S UTC").to_string();
                (j.clone(), time_str)
            })
        })
        .min_by_key(|(_, t)| t.clone());

    let next_scheduled = next.map(|(job, time)| NextScheduled { job, time });

    Ok(DiscoveryStats {
        total_jobs: total,
        active_jobs: active,
        failed_jobs: failed,
        schedulers_detected,
        next_scheduled,
        recent_executions: vec![],
    })
}

#[tauri::command]
pub fn trigger_discovery(state: State<AppState>) -> Result<(), String> {
    state.discovery.discover_all().map(|_| ())
}

#[tauri::command]
pub fn get_setting(state: State<AppState>, key: String) -> Result<Option<String>, String> {
    state.settings.get(&key).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_setting(state: State<AppState>, key: String, value: String) -> Result<(), String> {
    state.settings.set(&key, &value).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn toggle_job_enabled(state: State<AppState>, id: String) -> Result<Job, String> {
    let job = state
        .repo
        .get_job(&id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Job not found".to_string())?;

    let mut updated = job.clone();
    updated.enabled = !job.enabled;
    updated.updated_at = chrono::Utc::now();

    state.repo.upsert_job(&updated).map_err(|e| e.to_string())?;
    Ok(updated)
}

#[tauri::command]
pub fn remove_job(state: State<AppState>, id: String) -> Result<(), String> {
    state.repo.delete_job(&id).map_err(|e| e.to_string())
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveryStats {
    pub total_jobs: usize,
    pub active_jobs: usize,
    pub failed_jobs: usize,
    pub schedulers_detected: usize,
    pub next_scheduled: Option<NextScheduled>,
    pub recent_executions: Vec<Execution>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NextScheduled {
    pub job: Job,
    pub time: String,
}