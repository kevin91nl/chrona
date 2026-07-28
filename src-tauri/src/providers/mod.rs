pub mod interface;
pub mod codex;
pub mod cron;
pub mod launchd;
pub mod systemd;
pub mod windows_task;

pub mod future {
    //! Placeholder modules for future scheduler providers.
    //! Each will implement the SchedulerProvider trait.
    //!
    //! Planned:
    //! - GitHub Actions
    //! - Docker scheduled jobs
    //! - Kubernetes CronJobs
    //! - n8n workflows
    //! - Temporal workflows
}

pub use interface::SchedulerProvider;