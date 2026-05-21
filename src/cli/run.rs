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
    run_check_full(project_dir, format, ci_mode, only, None, false, None)
}

pub fn run_check_full(
    project_dir: &Path,
    format: ReportFormat,
    ci_mode: bool,
    only: Option<Vec<String>>,
    crate_name: Option<String>,
    changed_only: bool,
    set_cmd: Option<Vec<String>>,
) -> Result<()> {
    let config_path = project_dir.join(".rust-checker").join("config.toml");

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

    // Apply --set-cmd overrides: each entry is "TOOL=CMD"
    if let Some(overrides) = set_cmd {
        for item in overrides {
            if let Some((tool, cmd)) = item.split_once('=') {
                let tool = tool.trim().to_string();
                let cmd = cmd.trim().to_string();
                if let Some(tool_cfg) = config.tools.get_mut(&tool) {
                    tool_cfg.input_command = cmd;
                } else {
                    anyhow::bail!(
                        "--set-cmd 指定了未知工具 `{tool}`。\n\
                         请确认工具名称与配置文件中 [tools.<name>] 一致。"
                    );
                }
            } else {
                anyhow::bail!(
                    "--set-cmd 格式错误: `{item}`。正确格式为 TOOL=CMD，例如：\n\
                     --set-cmd clippy=\"cargo clippy -- -W clippy::all\""
                );
            }
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
        // --changed acts as a "change-guard": when there are no changed crates the run
        // is skipped entirely (saving time); when there are changes tools run at the
        // workspace root.  Per-crate filtering is not yet implemented.
        match crate::workspace::detect_workspace(project_dir) {
            None => {
                anyhow::bail!("当前项目不是 Cargo workspace，无法使用 --changed 选项");
            }
            Some(_) => {
                let changed = crate::workspace::get_changed_crates(project_dir)?;
                if changed.is_empty() {
                    println!("ℹ️  没有检测到变更的 crate，跳过检查。");
                    return Ok(());
                }
                println!("🔍 变更的 crate: {}", changed.join(", "));
                project_dir.to_path_buf()
            }
        }
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

    let runner = Runner::new(config, project_dir, &effective_dir, format, ci_mode);
    runner.run()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_check_full_missing_config_errors() {
        let dir = tempfile::tempdir().unwrap();
        let result = run_check_full(
            dir.path(),
            ReportFormat::Markdown,
            false,
            None,
            None,
            false,
            None,
        );
        assert!(result.is_err());
        let msg = format!("{}", result.unwrap_err());
        assert!(msg.contains("config") || msg.contains("init"));
    }

    #[test]
    fn test_set_cmd_bad_format_errors() {
        use std::io::Write;

        let dir = tempfile::tempdir().unwrap();
        let rc_dir = dir.path().join(".rust-checker");
        std::fs::create_dir_all(&rc_dir).unwrap();
        let mut f = std::fs::File::create(rc_dir.join("config.toml")).unwrap();
        write!(
            f,
            r#"schema_version = "2"
[tools.build]
desc = "build"
active = false
input_command = "cargo build"
"#
        )
        .unwrap();
        drop(f);

        // "no-equals" has no '=' separator → should error
        let result = run_check_full(
            dir.path(),
            ReportFormat::Markdown,
            false,
            None,
            None,
            false,
            Some(vec!["no-equals".to_string()]),
        );
        assert!(result.is_err());
        let msg = format!("{}", result.unwrap_err());
        assert!(msg.contains("格式错误") || msg.contains("TOOL=CMD"));
    }

    #[test]
    fn test_set_cmd_unknown_tool_errors() {
        use std::io::Write;

        let dir = tempfile::tempdir().unwrap();
        let rc_dir = dir.path().join(".rust-checker");
        std::fs::create_dir_all(&rc_dir).unwrap();
        let mut f = std::fs::File::create(rc_dir.join("config.toml")).unwrap();
        write!(
            f,
            r#"schema_version = "2"
[tools.build]
desc = "build"
active = false
input_command = "cargo build"
"#
        )
        .unwrap();
        drop(f);

        // "unknown_xyz=cargo test" references a tool not in config
        let result = run_check_full(
            dir.path(),
            ReportFormat::Markdown,
            false,
            None,
            None,
            false,
            Some(vec!["unknown_xyz=cargo test".to_string()]),
        );
        assert!(result.is_err());
        let msg = format!("{}", result.unwrap_err());
        assert!(msg.contains("unknown_xyz"));
    }

    #[test]
    fn test_set_cmd_overrides_tool_command() {
        use std::io::Write;

        let dir = tempfile::tempdir().unwrap();
        let rc_dir = dir.path().join(".rust-checker");
        std::fs::create_dir_all(&rc_dir).unwrap();
        let mut f = std::fs::File::create(rc_dir.join("config.toml")).unwrap();
        write!(
            f,
            r#"schema_version = "2"
[tools.build]
desc = "build"
active = false
input_command = "cargo build"
"#
        )
        .unwrap();
        drop(f);

        // A valid override for an existing tool should succeed (tool is inactive so no
        // actual cargo invocation happens, but the override itself must be accepted).
        let result = run_check_full(
            dir.path(),
            ReportFormat::Markdown,
            false,
            None,
            None,
            false,
            Some(vec!["build=cargo build --release".to_string()]),
        );
        // Should succeed (tool is inactive, so just skipped with new command stored)
        assert!(result.is_ok(), "unexpected error: {:?}", result.err());
    }
}
