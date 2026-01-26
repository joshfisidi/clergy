use clap::{Parser, Subcommand};

use crate::model::PurgeData;
use crate::safety;
use crate::shell;
use crate::ui;

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
                ui::run_ui()?;
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
    // Check cooldown
    match safety::can_run_purge() {
        Ok(()) => {}
        Err(remaining) => {
            println!(
                "Purge recently performed.\nPlease wait {} seconds before running again.",
                remaining.as_secs()
            );
            return Ok(());
        }
    }

    let data: PurgeData = shell::run_purge()?;
    safety::mark_purge_run();

    // JSON mode: machine-readable only
    if json {
        let output = serde_json::to_string_pretty(&data)?;
        println!("{output}");
        return Ok(());
    }

    // Headless mode: no RatATUI, plain text only
    if headless {
        render_headless(&data);
        return Ok(());
    }

    // Default: RatATUI interface
    ui::render_purge_ui(&data)?;
    Ok(())
}

/// Plain-text fallback rendering (no TUI)
fn render_headless(data: &PurgeData) {
    println!("CLERGY · SYSTEM PURGE");
    println!("Host      : {}", data.host);
    println!("User      : {}", data.user);
    println!("Started   : {}", data.start_time);
    println!("Duration  : {}s", data.duration_seconds);
    println!();

    println!("Disk (Before):");
    println!("{}", data.disk_before);
    println!();

    println!("Disk (After):");
    println!("{}", data.disk_after);
    println!();

    println!("DNS flushed        : {}", yes_no(data.dns_flushed));
    println!("Snapshots thinned : {}", yes_no(data.snapshots_thinned));
}

/// Helper for human-readable booleans
fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}
