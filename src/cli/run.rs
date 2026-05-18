use anyhow::{Context, Result};
use std::path::Path;

use crate::config::Config;
use crate::report::ReportFormat;
use crate::runner::Runner;

pub fn run_check(
    project_dir: &Path,
    format: ReportFormat,
    ci_mode: bool,
    only: Option<Vec<String>>,
) -> Result<()> {
    let config_path = project_dir.join(".localcheck").join("config.toml");

    if !config_path.exists() {
        anyhow::bail!(
            "配置文件不存在: {}\n请先运行 `rust-checker init` 生成配置文件",
            config_path.display()
        );
    }

    let mut config = Config::load(&config_path)
        .with_context(|| format!("加载配置文件失败: {}", config_path.display()))?;

    // Filter tools if --only is specified
    if let Some(only_tools) = only {
        config.tools.retain(|name, _| only_tools.contains(name));
        if config.tools.is_empty() {
            anyhow::bail!("未找到指定的工具: {:?}", only_tools);
        }
    }

    config.resolve_output_paths();

    println!("🔍 开始检查，共 {} 个工具", config.tools.len());

    let runner = Runner::new(config, project_dir, format, ci_mode);
    runner.run()
}
