mod commands;
mod db;
mod discovery;
mod models;
mod providers;

use db::{Database, JobRepository};
use discovery::DiscoveryEngine;
use providers::*;
use std::sync::Arc;
use tauri::Manager;

pub struct AppState {
    pub repo: Arc<JobRepository>,
    pub discovery: DiscoveryEngine,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // Determine database path
            let app_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to get app data dir");
            std::fs::create_dir_all(&app_dir).ok();

            let db_path = app_dir.join("chrona.db");
            let db = Database::new(&db_path).expect("Failed to open database");
            let db = Arc::new(db);
            let repo = Arc::new(JobRepository::new(db.clone()));

            // Register providers
            let providers: Vec<Box<dyn SchedulerProvider>> = vec![
                Box::new(providers::cron::CronProvider::new()),
                Box::new(providers::launchd::LaunchdProvider::new()),
                Box::new(providers::codex::CodexProvider::new()),
                // Box::new(providers::systemd::SystemdProvider::new()),
                // Box::new(providers::windows_task::WindowsTaskProvider::new()),
            ];

            let discovery = DiscoveryEngine::new(repo.clone(), providers);

            // Initial discovery
            match discovery.discover_all() {
                Ok(jobs) => log::info!("Initial discovery found {} jobs", jobs.len()),
                Err(e) => log::warn!("Initial discovery failed: {}", e),
            }

            app.manage(AppState {
                repo,
                discovery,
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_jobs,
            commands::get_job,
            commands::get_executions,
            commands::get_schedulers,
            commands::get_discovery_stats,
            commands::trigger_discovery,
        ])
        .run(tauri::generate_context!())
        .expect("Error running Chrona");
}