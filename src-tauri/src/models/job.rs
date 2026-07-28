use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Job {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub source: String,
    pub schedule: String,
    pub timezone: Option<String>,
    pub command: String,
    pub working_directory: Option<String>,
    pub status: JobStatus,
    pub enabled: bool,
    pub discovered_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub next_execution: Option<DateTime<Utc>>,
    pub previous_execution: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveredJob {
    pub name: String,
    pub provider: String,
    pub source: String,
    pub schedule: String,
    pub timezone: Option<String>,
    pub command: String,
    pub working_directory: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum JobStatus {
    Active,
    Inactive,
    Error,
    Running,
    Unknown,
}

impl std::fmt::Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobStatus::Active => write!(f, "active"),
            JobStatus::Inactive => write!(f, "inactive"),
            JobStatus::Error => write!(f, "error"),
            JobStatus::Running => write!(f, "running"),
            JobStatus::Unknown => write!(f, "unknown"),
        }
    }
}

impl From<&str> for JobStatus {
    fn from(s: &str) -> Self {
        match s {
            "active" => JobStatus::Active,
            "inactive" => JobStatus::Inactive,
            "error" => JobStatus::Error,
            "running" => JobStatus::Running,
            _ => JobStatus::Unknown,
        }
    }
}