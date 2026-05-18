use anyhow::Result;
use indexmap::IndexMap;
use std::path::Path;

use crate::config::{Config, RustConfig, ToolConfig};

/// Built-in tool presets grouped by category.
struct ToolPreset {
    name: &'static str,
    desc: &'static str,
    command: &'static str,
    deps: Option<&'static [&'static str]>,
}

const ALL_TOOLS: &[ToolPreset] = &[
    ToolPreset { name: "build",     desc: "构建项目",             command: "cargo build",                              deps: None },
    ToolPreset { name: "test",      desc: "运行单元测试",          command: "cargo test",                               deps: Some(&["build"]) },
    ToolPreset { name: "coverage",  desc: "测试覆盖率",            command: "cargo llvm-cov",                           deps: Some(&["build"]) },
    ToolPreset { name: "clippy",    desc: "代码静态分析",          command: "cargo clippy -- -D warnings",              deps: None },
    ToolPreset { name: "fmt",       desc: "代码格式检查",          command: "cargo fmt --check",                        deps: None },
    ToolPreset { name: "doc",       desc: "文档生成检查",          command: "cargo doc --no-deps",                      deps: None },
    ToolPreset { name: "audit",     desc: "安全漏洞审计",          command: "cargo audit",                              deps: None },
    ToolPreset { name: "deny",      desc: "依赖策略检查",          command: "cargo deny check",                         deps: None },
    ToolPreset { name: "geiger",    desc: "unsafe 代码检查",       command: "cargo geiger",                             deps: None },
    ToolPreset { name: "metrics",   desc: "代码指标统计",          command: "cargo geiger --output-format Ratio",       deps: None },
    ToolPreset { name: "deps",      desc: "依赖树展示",            command: "cargo tree",                               deps: None },
    ToolPreset { name: "msrv",      desc: "最低支持 Rust 版本",    command: "cargo msrv",                               deps: None },
    ToolPreset { name: "semver",    desc: "语义化版本检查",        command: "cargo semver-checks",                      deps: None },
    ToolPreset { name: "udeps",     desc: "未使用依赖检查",        command: "cargo +nightly udeps",                     deps: None },
    ToolPreset { name: "bench",     desc: "基准测试",              command: "cargo bench",                              deps: Some(&["build"]) },
    ToolPreset { name: "bloat",     desc: "二进制体积分析",        command: "cargo bloat --release",                    deps: None },
    ToolPreset { name: "flamegraph",desc: "性能火焰图",            command: "cargo flamegraph",                         deps: None },
    ToolPreset { name: "binary",    desc: "二进制信息",            command: "cargo build --release",                    deps: None },
];

pub fn run_init(project_dir: &Path, preset: &str, force: bool) -> Result<()> {
    let config_dir = project_dir.join(".localcheck");
    let config_path = config_dir.join("config.toml");

    if config_path.exists() && !force {
        println!("配置文件已存在: {}", config_path.display());
        println!("使用 --force 强制重新生成");
        return Ok(());
    }

    std::fs::create_dir_all(&config_dir)?;

    let tools = select_tools_by_preset(preset);
    let config = Config {
        schema_version: Some("1".to_string()),
        rust: Some(RustConfig {
            version: None,
            rustflags: Some(String::new()),
        }),
        tools,
    };

    let toml_str = toml::to_string_pretty(&config)?;
    std::fs::write(&config_path, toml_str)?;

    println!("✅ 配置文件已生成: {}", config_path.display());
    println!("预设: {preset}，包含 {} 个工具", config.tools.len());
    Ok(())
}

fn select_tools_by_preset(preset: &str) -> IndexMap<String, ToolConfig> {
    let selected_names: &[&str] = match preset {
        "minimal" => &["build", "test", "clippy", "fmt"],
        "quality" => &["build", "test", "clippy", "fmt", "doc", "coverage"],
        "security" => &["build", "test", "audit", "deny", "geiger"],
        "full" => &[
            "build", "test", "coverage", "clippy", "fmt", "doc",
            "audit", "deny", "geiger", "metrics", "deps", "msrv",
            "semver", "udeps", "bench", "bloat", "flamegraph", "binary",
        ],
        _ => &["build", "test", "clippy", "fmt"],
    };

    let mut tools = IndexMap::new();
    for preset_tool in ALL_TOOLS {
        if selected_names.contains(&preset_tool.name) {
            tools.insert(
                preset_tool.name.to_string(),
                ToolConfig {
                    desc: preset_tool.desc.to_string(),
                    active: true,
                    input_command: preset_tool.command.to_string(),
                    output_path: None,
                    depends_on: preset_tool.deps.map(|d| d.iter().map(|s| s.to_string()).collect()),
                },
            );
        }
    }
    tools
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_tools_minimal() {
        let tools = select_tools_by_preset("minimal");
        assert!(tools.contains_key("build"));
        assert!(tools.contains_key("test"));
        assert!(tools.contains_key("clippy"));
        assert!(tools.contains_key("fmt"));
        assert!(!tools.contains_key("audit"));
    }

    #[test]
    fn test_select_tools_full() {
        let tools = select_tools_by_preset("full");
        assert_eq!(tools.len(), ALL_TOOLS.len());
    }

    #[test]
    fn test_select_tools_unknown_preset() {
        let tools = select_tools_by_preset("unknown");
        // Falls back to default (build, test, clippy, fmt)
        assert!(tools.contains_key("build"));
    }
}
