//! CLI surface — non-TUI command interface
//!
//! This module handles command-line argument parsing and
//! headless execution modes.

mod commands;

pub use commands::{Cli, Commands};
