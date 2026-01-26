// src/safety.rs
//
// Safety guards for destructive or system-impacting operations.
// Currently enforces a cooldown between purge executions.

use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

/// Minimum time between purge runs
pub const PURGE_COOLDOWN: Duration = Duration::from_secs(60);

/// Last successful purge timestamp (process-local, thread-safe)
static LAST_PURGE: OnceLock<Mutex<Option<Instant>>> = OnceLock::new();

fn last_purge() -> &'static Mutex<Option<Instant>> {
    LAST_PURGE.get_or_init(|| Mutex::new(None))
}

/// Check whether a purge may be executed.
///
/// Returns:
/// - Ok(()) if allowed
/// - Err(remaining) if still in cooldown
pub fn can_run_purge() -> Result<(), Duration> {
    let guard = last_purge().lock().unwrap();

    if let Some(last) = *guard {
        let elapsed = last.elapsed();
        if elapsed < PURGE_COOLDOWN {
            return Err(PURGE_COOLDOWN - elapsed);
        }
    }
    Ok(())
}

/// Record a successful purge execution.
pub fn mark_purge_run() {
    let mut guard = last_purge().lock().unwrap();
    *guard = Some(Instant::now());
}
