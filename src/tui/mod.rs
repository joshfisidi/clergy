//! TUI runtime — main loop and orchestration
//!
//! This module handles:
//! - Terminal setup/teardown
//! - Main event loop
//! - State coordination
//!
//! It does NOT contain rendering (that's ui/) or input handling (that's input/).

mod run;

pub use run::{render_purge_ui, run_ui};
