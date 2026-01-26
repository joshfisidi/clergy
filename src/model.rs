use serde::{Deserialize, Serialize};

/// Structured output returned by the CLERGY shell backend.
///
/// This struct represents a *completed purge operation*.
/// It is intentionally:
/// - explicit
/// - human-readable
/// - UI-friendly
/// - stable across versions
///
/// Any future fields should be **additive only**.
#[derive(Debug, Deserialize, Serialize)]
pub struct PurgeData {
    /// Hostname of the system where the purge was executed
    pub host: String,

    /// User that initiated the purge
    pub user: String,

    /// Local start time of the purge (HH:MM:SS)
    pub start_time: String,

    /// Total duration of the purge in seconds
    pub duration_seconds: u64,

    /// Disk usage snapshot *before* purge (raw `df -h /` output)
    pub disk_before: String,

    /// Disk usage snapshot *after* purge (raw `df -h /` output)
    pub disk_after: String,

    /// Whether DNS caches were flushed successfully
    pub dns_flushed: bool,

    /// Whether Time Machine local snapshots were thinned
    pub snapshots_thinned: bool,
}
