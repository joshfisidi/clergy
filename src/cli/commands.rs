use clap::{Parser, Subcommand};

use crate::actions;
use crate::config;
use crate::model::PurgeData;
use crate::tui;

/// CLERGY — macOS daemon cleanup and cache invalidation utility
#[derive(Parser, Debug)]
#[command(
    name = "clergy",
    version,
    about = "Daemon cleanup, cache invalidation, and snapshot thinning for macOS",
    long_about = None
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Run a system purge (DNS cache + local snapshots)
    Purge {
        /// Output structured JSON only (no UI)
        #[arg(long)]
        json: bool,

        /// Run without TUI (human-readable text only)
        #[arg(long)]
        headless: bool,
    },
}

impl Cli {
    /// Execute the selected command
    pub fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        match self.command {
            None => {
                // Default: launch interactive TUI
                tui::run_ui()?;
            }
            Some(Commands::Purge { json, headless }) => {
                run_purge(json, headless)?;
            }
        }
        Ok(())
    }
}

/// Executes the purge command with the requested output mode
fn run_purge(json: bool, headless: bool) -> Result<(), Box<dyn std::error::Error>> {
    let settings = config::load();

    // Check cooldown
    match actions::can_run_purge(&settings) {
        Ok(()) => {}
        Err(remaining) => {
            println!(
                "Purge recently performed.\nPlease wait {} seconds before running again.",
                remaining.as_secs()
            );
            return Ok(());
        }
    }

    actions::authenticate()?;
    let mut data: PurgeData = actions::run_purge()?;
    if data.completed() {
        actions::mark_purge_run();
    }
    if let Err(error) = actions::save_last(&data) {
        data.report_save_error = Some(error.to_string());
    }

    // JSON mode: machine-readable only
    if json {
        let output = serde_json::to_string_pretty(&data)?;
        println!("{output}");
        return finish_status(&data);
    }

    // Headless mode: no RatATUI, plain text only
    if headless {
        render_headless(&data);
        return finish_status(&data);
    }

    // Default: RatATUI interface
    tui::render_purge_ui(&data)?;
    finish_status(&data)
}

fn finish_status(data: &PurgeData) -> Result<(), Box<dyn std::error::Error>> {
    if data.completed() {
        Ok(())
    } else {
        Err("Purge incomplete; see the action journal for failures and skipped steps.".into())
    }
}

/// Plain-text fallback rendering (no TUI)
fn render_headless(data: &PurgeData) {
    for line in crate::ui::report_lines(data, 90, &crate::theme::Theme::default()) {
        println!("{line}");
    }
}
