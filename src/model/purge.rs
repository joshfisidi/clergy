use serde::{Deserialize, Serialize};

/// Memory statistics from vm_stat
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MemoryStats {
    pub used_mb: u64,
    pub free_mb: u64,
    pub compressed_mb: u64,
}

/// Swap statistics from sysctl vm.swapusage
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SwapStats {
    pub used_mb: u64,
    pub free_mb: u64,
}

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
#[derive(Debug, Deserialize, Serialize, Clone)]
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

    /// Memory stats before purge
    pub memory_before: MemoryStats,

    /// Memory stats after purge
    pub memory_after: MemoryStats,

    /// Swap stats before purge
    pub swap_before: SwapStats,

    /// Swap stats after purge
    pub swap_after: SwapStats,

    /// Whether DNS caches were flushed successfully
    pub dns_flushed: bool,

    /// Whether Time Machine local snapshots were thinned
    pub snapshots_thinned: bool,

    /// Full local timestamps and measured command outcomes (absent in old reports).
    #[serde(default)]
    pub started_at: Option<String>,
    #[serde(default)]
    pub finished_at: Option<String>,
    #[serde(default)]
    pub actions: Vec<ActionResult>,
    /// Exact df -k availability samples; do not infer reclaimed bytes from df -h.
    #[serde(default)]
    pub disk_available_before_kib: Option<u64>,
    #[serde(default)]
    pub disk_available_after_kib: Option<u64>,
    /// None means enumeration failed, while an empty list means zero snapshots.
    #[serde(default)]
    pub snapshots_before: Option<Vec<String>>,
    #[serde(default)]
    pub snapshots_after: Option<Vec<String>>,
    #[serde(default)]
    pub report_save_error: Option<String>,
    /// New reports distinguish failed telemetry reads from real zero values.
    #[serde(default)]
    pub memory_before_available: Option<bool>,
    #[serde(default)]
    pub memory_after_available: Option<bool>,
    #[serde(default)]
    pub swap_before_available: Option<bool>,
    #[serde(default)]
    pub swap_after_available: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ActionStatus {
    Succeeded,
    Failed,
    Skipped,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ActionResult {
    pub label: String,
    pub command: String,
    pub status: ActionStatus,
    pub duration_ms: u64,
    pub exit_code: Option<i32>,
    #[serde(default)]
    pub output: String,
}

impl PurgeData {
    pub fn completed(&self) -> bool {
        self.dns_flushed
            && self.snapshots_thinned
            && self
                .actions
                .iter()
                .all(|action| action.status == ActionStatus::Succeeded)
    }

    pub fn removed_snapshots(&self) -> Option<Vec<&str>> {
        let before = self.snapshots_before.as_ref()?;
        let after = self.snapshots_after.as_ref()?;
        Some(
            before
                .iter()
                .filter(|name| !after.contains(name))
                .map(String::as_str)
                .collect(),
        )
    }
}
