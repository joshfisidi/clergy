use clap::Parser;

mod about;
mod cli;
mod model;
mod safety;
mod shell;
mod ui;

use cli::Cli;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().count();

    // NO ARGS = INTERACTIVE MODE
    if args == 1 {
        return ui::run_ui();
    }

    // Otherwise use CLI
    let cli = Cli::parse();
    cli.run()
}
