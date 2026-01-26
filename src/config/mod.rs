//! User configuration — persistent settings
//!
//! This module contains user-facing configuration that is:
//! - Serializable to disk
//! - Human-readable (TOML)
//! - Not runtime state

mod settings;

pub use settings::{
    load, save, settings_path, ColorName, PurgeSettings, Settings, ThemeKind, ThemeOverrides,
};
