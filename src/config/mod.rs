use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use anyhow::{Context, Result};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub schema_version: Option<String>,
    pub rust: Option<RustConfig>,
    #[serde(default)]
    pub tools: IndexMap<String, ToolConfig>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RustConfig {
    pub version: Option<String>,
    pub rustflags: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ToolConfig {
    pub desc: String,
    pub active: bool,
    pub input_command: String,
    pub output_path: Option<String>,
    pub depends_on: Option<Vec<String>>,
}

pub fn builtin_output_path(tool_name: &str) -> Option<&'static str> {
    match tool_name {
        "build" => Some("quality/build.md"),
        "test" => Some("quality/test.md"),
        "coverage" => Some("quality/coverage.md"),
        "metrics" => Some("perf/metrics.md"),
        "clippy" => Some("quality/clippy.md"),
        "fmt" => Some("quality/fmt.md"),
        "doc" => Some("quality/doc.md"),
        "audit" => Some("security/audit.md"),
        "deny" => Some("security/deny.md"),
        "geiger" => Some("security/geiger.md"),
        "deps" => Some("deps/deps.md"),
        "msrv" => Some("compat/msrv.md"),
        "semver" => Some("compat/semver.md"),
        "udeps" => Some("deps/udeps.md"),
        "bench" => Some("perf/bench.md"),
        "bloat" => Some("perf/bloat.md"),
        "flamegraph" => Some("perf/flamegraph.md"),
        "binary" => Some("compat/binary.md"),
        _ => None,
    }
}

pub fn effective_output_path(tool_name: &str, configured: Option<&str>) -> String {
    if let Some(p) = configured {
        return p.to_string();
    }
    if let Some(p) = builtin_output_path(tool_name) {
        return p.to_string();
    }
    format!("customs/{tool_name}.md")
}

impl Config {
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config: {}", path.display()))?;
        let config: Config = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config: {}", path.display()))?;
        Ok(config)
    }

    pub fn resolve_output_paths(&mut self) {
        for (name, tool) in &mut self.tools {
            if tool.output_path.is_none() {
                tool.output_path = Some(effective_output_path(name, None));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_output_paths() {
        assert_eq!(builtin_output_path("build"), Some("quality/build.md"));
        assert_eq!(builtin_output_path("test"), Some("quality/test.md"));
        assert_eq!(builtin_output_path("audit"), Some("security/audit.md"));
        assert_eq!(builtin_output_path("unknown_tool"), None);
    }

    #[test]
    fn test_effective_output_path_custom() {
        assert_eq!(effective_output_path("mytool", None), "customs/mytool.md");
    }

    #[test]
    fn test_effective_output_path_configured() {
        assert_eq!(
            effective_output_path("build", Some("custom/path.md")),
            "custom/path.md"
        );
    }

    #[test]
    fn test_config_parse() {
        let toml_str = r#"
schema_version = "1"

[rust]
version = "1.75.0"
rustflags = ""

[tools.build]
desc = "构建项目"
active = true
input_command = "cargo build"
output_path = "quality/build.md"

[tools.test]
desc = "运行单元测试"
active = true
input_command = "cargo test"
depends_on = ["build"]
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.schema_version.as_deref(), Some("1"));
        assert_eq!(config.tools.len(), 2);
        assert!(config.tools.contains_key("build"));
        assert_eq!(
            config.tools["test"].depends_on.as_deref(),
            Some(["build".to_string()].as_slice())
        );
    }
}
