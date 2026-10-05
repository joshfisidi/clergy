//! Data models — shared types across the application
//!
//! These are pure data structures with no behavior.

mod purge;

pub use purge::{ActionResult, ActionStatus, MemoryStats, PurgeData, SwapStats};
