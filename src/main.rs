use clap::{Parser, Subcommand};
use std::path::PathBuf;

use rust_checker::cli;
use rust_checker::report::ReportFormat;

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

        /// 只检查指定 crate（Workspace 模式）
        #[arg(long = "crate", value_name = "CRATE")]
        crate_name: Option<String>,

        /// 只在本次 git diff 有 crate 变更时运行检查（Workspace 模式；无变更时跳过）
        #[arg(long)]
        changed: bool,
    },

    /// 查看两次检查结果的差异或历史趋势
    Diff {
        /// 项目目录（默认当前目录）
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,

        /// 展示最近 N 次趋势
        #[arg(long, value_name = "N")]
        last: Option<usize>,

        /// 时间范围起始（YYYYMMDD 格式）
        #[arg(long, value_name = "YYYYMMDD")]
        from: Option<String>,

        /// 时间范围结束（YYYYMMDD 格式）
        #[arg(long, value_name = "YYYYMMDD")]
        to: Option<String>,
    },

    /// 管理插件
    Plugin {
        #[command(subcommand)]
        action: PluginAction,
    },

    /// 监听文件变更并自动重跑检查
    Watch {
        /// 项目目录（默认当前目录）
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,

        /// 只重跑指定工具（逗号分隔）
        #[arg(long, value_delimiter = ',')]
        tools: Option<Vec<String>>,
    },

    /// 将配置文件升级到最新 schema 版本
    Upgrade {
        /// 项目目录（默认当前目录）
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
    },
}

#[derive(Subcommand)]
enum PluginAction {
    /// 列出已安装的插件
    List {
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
    },
    /// 从注册表安装插件
    Add {
        /// 插件名称
        name: String,
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
    },
    /// 卸载插件
    Remove {
        /// 插件名称
        name: String,
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
    },
    /// 更新所有已安装的插件
    Update {
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { dir, preset, force } => {
            cli::init::run_init(&dir, &preset, force)?;
        }

        Commands::Run {
            dir,
            format,
            ci,
            only,
            crate_name,
            changed,
        } => {
            let report_format = match format.as_str() {
                "html" => ReportFormat::Html,
                "json" => ReportFormat::Json,
                _ => ReportFormat::Markdown,
            };
            cli::run::run_check_full(&dir, report_format, ci, only, crate_name, changed)?;
        }

        Commands::Diff {
            dir,
            last,
            from,
            to,
        } => {
            let mode = if let Some(n) = last {
                cli::diff::DiffMode::Last(n)
            } else if from.is_some() || to.is_some() {
                let from = from.unwrap_or_else(|| "00000000".to_string());
                let to = to.unwrap_or_else(|| "99999999".to_string());
                cli::diff::DiffMode::Range { from, to }
            } else {
                cli::diff::DiffMode::Latest
            };
            cli::diff::run_diff(&dir, mode)?;
        }

        Commands::Plugin { action } => match action {
            PluginAction::List { dir } => cli::plugin::run_plugin_list(&dir)?,
            PluginAction::Add { name, dir } => cli::plugin::run_plugin_add(&name, &dir)?,
            PluginAction::Remove { name, dir } => cli::plugin::run_plugin_remove(&name, &dir)?,
            PluginAction::Update { dir } => cli::plugin::run_plugin_update(&dir)?,
        },

        Commands::Watch { dir, tools } => {
            cli::watch::run_watch(&dir, tools)?;
        }

        Commands::Upgrade { dir } => {
            cli::upgrade::run_upgrade(&dir)?;
        }
    }

    Ok(())
}
