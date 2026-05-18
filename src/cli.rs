use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "rust-checker", about = "A unified Rust project quality checker", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Init(InitArgs),
    Run(RunArgs),
    Diff(DiffArgs),
    Upgrade,
    Plugin(PluginArgs),
    Watch(WatchArgs),
}

#[derive(Args)]
pub struct InitArgs {
    #[arg(long, value_enum)]
    pub preset: Option<Preset>,
    #[arg(long)]
    pub interactive: bool,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum Preset {
    Minimal,
    Standard,
    Full,
}

#[derive(Args)]
pub struct RunArgs {
    #[arg(long, value_enum, default_value = "markdown")]
    pub format: ReportFormat,
    #[arg(long)]
    pub ci: bool,
    #[arg(long)]
    pub crate_name: Option<String>,
    #[arg(long)]
    pub changed: bool,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum ReportFormat {
    Markdown,
    Html,
    Json,
}

#[derive(Args)]
pub struct DiffArgs {
    #[arg(long)]
    pub from: Option<String>,
    #[arg(long)]
    pub to: Option<String>,
    #[arg(long)]
    pub last: Option<usize>,
    #[arg(long, value_enum, default_value = "markdown")]
    pub format: ReportFormat,
}

#[derive(Args)]
pub struct PluginArgs {
    #[command(subcommand)]
    pub action: PluginAction,
}

#[derive(Subcommand)]
pub enum PluginAction {
    List,
    Add { name: String },
    Remove { name: String },
    Update,
}

#[derive(Args)]
pub struct WatchArgs {
    #[arg(long, value_delimiter = ',')]
    pub tools: Option<Vec<String>>,
}
