use crate::models::{DiscoveredJob, Execution};
use std::fmt;

/// Core trait that every scheduler provider must implement.
///
/// Providers are responsible for:
/// - Discovering jobs from their native scheduler
/// - Reading metadata
/// - Collecting execution information
/// - Translating native concepts into Chrona's normalized format
pub trait SchedulerProvider: Send + Sync {
    /// Unique identifier (e.g. "cron", "launchd", "systemd")
    fn id(&self) -> &str;

    /// Human-readable name (e.g. "cron", "launchd", "systemd timers")
    fn name(&self) -> &str;

    /// Whether this provider is available on the current OS
    fn is_available(&self) -> bool;

    /// Initial discovery — scan for all jobs
    fn discover(&self) -> Result<Vec<DiscoveredJob>, ProviderError>;

    /// Incremental refresh — detect new, changed, removed jobs
    fn refresh(&self) -> Result<Vec<DiscoveredJob>, ProviderError>;

    /// Whether this provider supports live execution history
    fn supports_execution_history(&self) -> bool {
        false
    }

    /// Get execution history for a specific job (if supported)
    fn get_execution_history(&self, _job_id: &str) -> Result<Vec<Execution>, ProviderError> {
        Ok(vec![])
    }
}

#[derive(Debug)]
pub enum ProviderError {
    Io(std::io::Error),
    Parse(String),
    NotFound(String),
    PermissionDenied(String),
    Other(String),
}

impl fmt::Display for ProviderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProviderError::Io(e) => write!(f, "IO error: {}", e),
            ProviderError::Parse(msg) => write!(f, "Parse error: {}", msg),
            ProviderError::NotFound(msg) => write!(f, "Not found: {}", msg),
            ProviderError::PermissionDenied(msg) => {
                write!(f, "Permission denied: {}", msg)
            }
            ProviderError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for ProviderError {}

impl From<std::io::Error> for ProviderError {
    fn from(e: std::io::Error) -> Self {
        ProviderError::Io(e)
    }
}