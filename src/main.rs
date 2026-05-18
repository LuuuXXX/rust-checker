use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod cli;
mod config;
mod logger;
mod report;
mod runner;
mod tools;

use crate::report::ReportFormat;

#[derive(Parser)]
#[command(
    name = "rust-checker",
    version,
    about = "Rust 项目质量检查工具",
    long_about = "rust-checker 自动运行多种 Rust 工具并生成统一报告"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 初始化配置文件
    Init {
        /// 项目目录（默认当前目录）
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,

        /// 预设配置 (minimal | quality | security | full)
        #[arg(short, long, default_value = "minimal")]
        preset: String,

        /// 强制重新生成（覆盖已有配置）
        #[arg(long)]
        force: bool,
    },

    /// 运行检查
    Run {
        /// 项目目录（默认当前目录）
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,

        /// 报告格式 (markdown | html | json)
        #[arg(short, long, default_value = "markdown")]
        format: String,

        /// CI 模式：跳过交互提示，额外输出 JSON 结果
        #[arg(long)]
        ci: bool,

        /// 只运行指定的工具（逗号分隔）
        #[arg(long, value_delimiter = ',')]
        only: Option<Vec<String>>,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { dir, preset, force } => {
            cli::init::run_init(&dir, &preset, force)?;
        }
        Commands::Run { dir, format, ci, only } => {
            let report_format = match format.as_str() {
                "html" => ReportFormat::Html,
                "json" => ReportFormat::Json,
                _ => ReportFormat::Markdown,
            };
            cli::run::run_check(&dir, report_format, ci, only)?;
        }
    }

    Ok(())
}
