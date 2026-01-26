//! About section for the TUI application.

/// Version from Cargo.toml
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// About text lines
pub const ABOUT_TEXT: &[&str] = &[
    "CLERGY is a local macOS system utility.",
    "",
    "It safely performs:",
    "• DNS cache invalidation",
    "• Daemon refresh",
    "• Time Machine snapshot thinning",
    "",
    "CLERGY does NOT:",
    "• Kill apps",
    "• Modify user files",
    "• Run background services",
    "",
    "All actions are explicit, local, and reversible.",
];

/// Footer attribution
pub const ATTRIBUTION: &[&str] = &[
    "Fisidi Labs",
    "A division of Fisidi Inc.",
];

pub fn about_text() -> &'static [&'static str] {
    ABOUT_TEXT
}

pub fn attribution() -> &'static [&'static str] {
    ATTRIBUTION
}
