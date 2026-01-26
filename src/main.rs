use clap::Parser;

// ─────────────────────────────────────────────────────────────
// Module declarations (organized by domain)
// ─────────────────────────────────────────────────────────────

// Runtime & orchestration
mod app;
mod tui;

// User interface
mod input;
mod layout;
mod theme;
mod ui;

// System telemetry
mod system;

// Actions (what CLERGY does)
mod actions;

// Configuration
mod config;

// Data models
mod model;

// Branding
mod branding;

// CLI surface
mod cli;

// Background tasks (may move to actions/ later)
mod worker;

use cli::Cli;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().count();

    // NO ARGS = INTERACTIVE MODE
    if args == 1 {
        return tui::run_ui();
    }

    // Otherwise use CLI
    let cli = Cli::parse();
    cli.run()
}
