mod cli;
mod commands;
mod config;
mod error;
mod io;
mod perf;
mod print;
mod tui;
mod utils;

use anyhow::Result;
use clap::Parser;

// Re-export for convenience
pub use config::Config;
pub use error::{CliError, Result as CliResult};
pub use io::*;
pub use print::Printer;

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {:#}", err);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = cli::Cli::parse();

    match cli.command {
        cli::Commands::Container(cmd) => cmd.execute(),
        cli::Commands::Codec(cmd) => cmd.execute(),
        cli::Commands::Embedding(cmd) => cmd.execute(),
        cli::Commands::Spatial(cmd) => cmd.execute(),
        cli::Commands::Quant(cmd) => cmd.execute(),
        cli::Commands::Transport(cmd) => cmd.execute(),
        cli::Commands::Envelope(cmd) => cmd.execute(),
        cli::Commands::Convert(cmd) => cmd.execute(),
        cli::Commands::Info(cmd) => cmd.execute(),
        cli::Commands::Validate(cmd) => cmd.execute(),
        cli::Commands::Perf(cmd) => cmd.execute(),
        cli::Commands::Tui => {
            let mut app = tui::App::new();
            app.run()
        }
    }
}
