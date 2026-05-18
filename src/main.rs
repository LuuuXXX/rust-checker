mod cli;
mod config;
mod commands;
mod engine;
mod tools;
mod report;
mod history;
mod logger;

use anyhow::Result;
use cli::{Cli, Commands};
use clap::Parser;

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init(args) => commands::init::run(args),
        Commands::Run(args) => commands::run::run(args),
        Commands::Diff(args) => commands::diff::run(args),
        Commands::Upgrade => commands::upgrade::run(),
        Commands::Plugin(args) => commands::plugin::run(args),
        Commands::Watch(args) => commands::watch::run(args),
    }
}
