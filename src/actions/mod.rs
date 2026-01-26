//! Actions — things CLERGY actually does
//!
//! This module contains the effectful operations:
//! - Running the purge
//! - Safety guards (cooldowns)
//! - Persisting results

mod purge;
mod results;
mod safety;

pub use purge::{run_purge, ShellError};
pub use results::{load_last, save_last};
pub use safety::{can_run_purge, mark_purge_run};
