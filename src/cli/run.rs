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
    run_check_full(project_dir, format, ci_mode, only, None, false)
}

pub fn run_check_full(
    project_dir: &Path,
    format: ReportFormat,
    ci_mode: bool,
    only: Option<Vec<String>>,
    crate_name: Option<String>,
    changed_only: bool,
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

    // Merge installed plugin tools into config (unless already defined)
    if let Ok(plugin_tools) = crate::plugin::load_plugin_tools(project_dir) {
        for (name, tool_cfg) in plugin_tools {
            config.tools.entry(name).or_insert(tool_cfg);
        }
    }

    // Determine the effective project directory (workspace member or root)
    let effective_dir = if let Some(ref crate_nm) = crate_name {
        // Workspace --crate mode
        match crate::workspace::detect_workspace(project_dir) {
            Some(ws) => {
                let member = ws.members.iter().find(|m| &m.name == crate_nm);
                match member {
                    Some(m) => m.path.clone(),
                    None => {
                        anyhow::bail!("工作区中未找到 crate: `{crate_nm}`");
                    }
                }
            }
            None => {
                anyhow::bail!("当前项目不是 Cargo workspace，无法使用 --crate 选项");
            }
        }
    } else if changed_only {
        // --changed: only run tools for changed crates
        let changed = crate::workspace::get_changed_crates(project_dir)?;
        if changed.is_empty() {
            println!("ℹ️  没有检测到变更的 crate，跳过检查。");
            return Ok(());
        }
        println!("🔍 变更的 crate: {}", changed.join(", "));
        project_dir.to_path_buf()
    } else {
        project_dir.to_path_buf()
    };

    // Filter tools if --only is specified
    if let Some(only_tools) = only {
        config.tools.retain(|name, _| only_tools.contains(name));
        if config.tools.is_empty() {
            anyhow::bail!("未找到指定的工具: {:?}", only_tools);
        }
    }

    config.resolve_output_paths();

    println!("🔍 开始检查，共 {} 个工具", config.tools.len());

    let runner = Runner::new(config, &effective_dir, format, ci_mode);
    runner.run()
}
