use anyhow::Result;
use std::path::Path;

use crate::watch::{build_options, watch};

pub fn run_watch(project_dir: &Path, tools: Option<Vec<String>>) -> Result<()> {
    let config_path = project_dir.join(".rust-checker").join("config.toml");

    if !config_path.exists() {
        anyhow::bail!(
            "配置文件不存在: {}\n请先运行 `rust-checker init`",
            config_path.display()
        );
    }

    let config = crate::config::Config::load(&config_path)?;

    let mut watch_opts = build_options(config.watch.as_ref(), project_dir);

    // Command-line --tools overrides config
    if tools.is_some() {
        watch_opts.tools = tools;
    }

    let project_dir_owned = project_dir.to_path_buf();

    watch(watch_opts, move |only| {
        crate::cli::run::run_check(
            &project_dir_owned,
            crate::report::ReportFormat::Markdown,
            false,
            only,
        )
    })
}
