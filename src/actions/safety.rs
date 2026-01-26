// src/safety.rs
//
// Safety guards for destructive or system-impacting operations.
// Currently enforces a cooldown between purge executions.

use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use crate::config::Settings;

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
pub fn can_run_purge(settings: &Settings) -> Result<(), Duration> {
    let cooldown = settings.purge_cooldown();
    let guard = last_purge().lock().unwrap();

    if let Some(last) = *guard {
        let elapsed = last.elapsed();
        if elapsed < cooldown {
            return Err(cooldown - elapsed);
        }
    }
    Ok(())
}

/// Record a successful purge execution.
pub fn mark_purge_run() {
    let mut guard = last_purge().lock().unwrap();
    *guard = Some(Instant::now());
}
