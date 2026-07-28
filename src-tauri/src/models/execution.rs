use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Execution {
    pub id: String,
    pub job_id: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration_ms: Option<i64>,
    pub status: ExecutionStatus,
    pub exit_code: Option<i32>,
    pub logs: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ExecutionStatus {
    Success,
    Failure,
    Running,
    Skipped,
    Timeout,
}

impl std::fmt::Display for ExecutionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutionStatus::Success => write!(f, "success"),
            ExecutionStatus::Failure => write!(f, "failure"),
            ExecutionStatus::Running => write!(f, "running"),
            ExecutionStatus::Skipped => write!(f, "skipped"),
            ExecutionStatus::Timeout => write!(f, "timeout"),
        }
    }
}

impl From<&str> for ExecutionStatus {
    fn from(s: &str) -> Self {
        match s {
            "success" => ExecutionStatus::Success,
            "failure" => ExecutionStatus::Failure,
            "running" => ExecutionStatus::Running,
            "skipped" => ExecutionStatus::Skipped,
            "timeout" => ExecutionStatus::Timeout,
            _ => ExecutionStatus::Success,
        }
    }
}