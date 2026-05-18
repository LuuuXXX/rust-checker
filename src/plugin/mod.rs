use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Base URL for the official plugin registry.
pub const REGISTRY_BASE_URL: &str =
    "https://raw.githubusercontent.com/LuuuXXX/rust-checker-plugins/main";

// ─── plugin.toml schema ──────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PluginMeta {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: Option<String>,
    pub category: String,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PluginCommand {
    pub program: String,
    pub args: Vec<String>,
    pub env: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PluginReport {
    pub parser: String,
    pub output_path: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PluginToml {
    pub plugin: PluginMeta,
    pub command: PluginCommand,
    pub report: PluginReport,
}

impl PluginToml {
    /// Build the CLI command string (`program args…`).
    pub fn command_string(&self) -> String {
        if self.command.args.is_empty() {
            self.command.program.clone()
        } else {
            format!("{} {}", self.command.program, self.command.args.join(" "))
        }
    }
}

// ─── directory helpers ───────────────────────────────────────────────────────

/// Return the plugins directory for a project: `.localcheck/plugins`.
pub fn plugins_dir(project_dir: &Path) -> PathBuf {
    project_dir.join(".localcheck").join("plugins")
}

// ─── install / remove / list / update ────────────────────────────────────────

/// Download and install a plugin from the official registry.
pub fn install_plugin(name: &str, project_dir: &Path) -> Result<()> {
    let url = format!("{REGISTRY_BASE_URL}/plugins/{name}/plugin.toml");

    let response = ureq::get(&url)
        .call()
        .with_context(|| format!("无法从注册表下载插件 `{name}` (URL: {url})"))?;

    let content = response
        .into_string()
        .with_context(|| format!("读取插件 `{name}` 响应内容失败"))?;

    // Validate before saving
    toml::from_str::<PluginToml>(&content)
        .with_context(|| format!("插件 `{name}` 的 plugin.toml 格式无效"))?;

    let dir = plugins_dir(project_dir).join(name);
    std::fs::create_dir_all(&dir)?;
    std::fs::write(dir.join("plugin.toml"), &content)?;

    println!("✅ 已安装插件: {name}");
    Ok(())
}

/// Remove an installed plugin.
pub fn remove_plugin(name: &str, project_dir: &Path) -> Result<()> {
    let dir = plugins_dir(project_dir).join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir)?;
        println!("✅ 已卸载插件: {name}");
    } else {
        println!("⚠️  插件未安装: {name}");
    }
    Ok(())
}

/// List all installed plugins.
pub fn list_plugins(project_dir: &Path) -> Result<Vec<PluginToml>> {
    let dir = plugins_dir(project_dir);
    if !dir.exists() {
        return Ok(vec![]);
    }

    let mut plugins: Vec<PluginToml> = Vec::new();
    let mut entries: Vec<_> = std::fs::read_dir(&dir)?.filter_map(|e| e.ok()).collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let plugin_path = entry.path().join("plugin.toml");
        if !plugin_path.exists() {
            continue;
        }
        let content = std::fs::read_to_string(&plugin_path)?;
        match toml::from_str::<PluginToml>(&content) {
            Ok(p) => plugins.push(p),
            Err(e) => {
                eprintln!("⚠️  跳过无效插件 {:?}: {}", entry.path(), e);
            }
        }
    }

    Ok(plugins)
}

/// Update all installed plugins by re-downloading them from the registry.
pub fn update_plugins(project_dir: &Path) -> Result<()> {
    let plugins = list_plugins(project_dir)?;
    if plugins.is_empty() {
        println!("没有已安装的插件");
        return Ok(());
    }

    let names: Vec<String> = plugins.iter().map(|p| p.plugin.name.clone()).collect();
    for name in &names {
        println!("更新插件: {name}...");
        install_plugin(name, project_dir)?;
    }

    println!("✅ 所有 {} 个插件已更新", names.len());
    Ok(())
}

/// Convert installed plugins to `(name, ToolConfig)` pairs for the runner.
pub fn load_plugin_tools(project_dir: &Path) -> Result<Vec<(String, crate::config::ToolConfig)>> {
    let plugins = list_plugins(project_dir)?;
    let mut tools = Vec::new();

    for plugin in plugins {
        let tool_config = crate::config::ToolConfig {
            desc: plugin.plugin.description.clone(),
            active: true,
            input_command: plugin.command_string(),
            output_path: Some(plugin.report.output_path.clone()),
            depends_on: None,
        };
        tools.push((plugin.plugin.name.clone(), tool_config));
    }

    Ok(tools)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_plugin_toml() -> &'static str {
        r#"
[plugin]
name        = "test-plugin"
version     = "0.1.0"
description = "A test plugin"
author      = "test"
category    = "quality"
tags        = ["test"]

[command]
program = "cargo"
args    = ["test"]

[report]
parser      = "builtin::test"
output_path = "quality/test.md"
"#
    }

    #[test]
    fn test_plugin_toml_parse() {
        let p: PluginToml = toml::from_str(sample_plugin_toml()).unwrap();
        assert_eq!(p.plugin.name, "test-plugin");
        assert_eq!(p.plugin.version, "0.1.0");
        assert_eq!(p.command.program, "cargo");
        assert_eq!(p.command.args, vec!["test"]);
        assert_eq!(p.report.output_path, "quality/test.md");
    }

    #[test]
    fn test_plugin_command_string() {
        let p: PluginToml = toml::from_str(sample_plugin_toml()).unwrap();
        assert_eq!(p.command_string(), "cargo test");
    }

    #[test]
    fn test_list_plugins_empty() {
        let dir = tempfile::tempdir().unwrap();
        let plugins = list_plugins(dir.path()).unwrap();
        assert!(plugins.is_empty());
    }

    #[test]
    fn test_remove_nonexistent_plugin_is_ok() {
        let dir = tempfile::tempdir().unwrap();
        // Should not return an error
        remove_plugin("nonexistent", dir.path()).unwrap();
    }

    #[test]
    fn test_install_and_list_local() {
        // Simulate install by writing plugin.toml directly
        let dir = tempfile::tempdir().unwrap();
        let plugin_dir = plugins_dir(dir.path()).join("clippy");
        std::fs::create_dir_all(&plugin_dir).unwrap();
        std::fs::write(plugin_dir.join("plugin.toml"), sample_plugin_toml()).unwrap();

        let plugins = list_plugins(dir.path()).unwrap();
        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0].plugin.name, "test-plugin");
    }

    #[test]
    fn test_remove_installed_plugin() {
        let dir = tempfile::tempdir().unwrap();
        let plugin_dir = plugins_dir(dir.path()).join("myplugin");
        std::fs::create_dir_all(&plugin_dir).unwrap();
        std::fs::write(plugin_dir.join("plugin.toml"), sample_plugin_toml()).unwrap();

        remove_plugin("myplugin", dir.path()).unwrap();
        assert!(!plugin_dir.exists());
    }

    #[test]
    fn test_load_plugin_tools() {
        let dir = tempfile::tempdir().unwrap();
        let plugin_dir = plugins_dir(dir.path()).join("myplugin");
        std::fs::create_dir_all(&plugin_dir).unwrap();
        std::fs::write(plugin_dir.join("plugin.toml"), sample_plugin_toml()).unwrap();

        let tools = load_plugin_tools(dir.path()).unwrap();
        assert_eq!(tools.len(), 1);
        let (name, cfg) = &tools[0];
        assert_eq!(name, "test-plugin");
        assert_eq!(cfg.input_command, "cargo test");
        assert_eq!(cfg.output_path.as_deref(), Some("quality/test.md"));
        assert!(cfg.active);
    }
}
