use anyhow::{Context, Result};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub schema_version: Option<String>,
    pub rust: Option<RustConfig>,
    pub history: Option<HistoryConfig>,
    pub watch: Option<WatchConfig>,
    #[serde(default)]
    pub tools: IndexMap<String, ToolConfig>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct HistoryConfig {
    pub max_entries: Option<u32>,
}

impl HistoryConfig {
    pub fn max_entries(&self) -> u32 {
        self.max_entries.unwrap_or(10)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WatchConfig {
    pub paths: Option<Vec<String>>,
    pub debounce_ms: Option<u64>,
    pub tools: Option<Vec<String>>,
}

impl WatchConfig {
    pub fn debounce_ms(&self) -> u64 {
        self.debounce_ms.unwrap_or(500)
    }
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
        "valgrind_memcheck" => Some("valgrind/memcheck.md"),
        "valgrind_helgrind" => Some("valgrind/helgrind.md"),
        "valgrind_drd" => Some("valgrind/drd.md"),
        "asan" => Some("asan.md"),
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
    fn test_builtin_output_paths_all_known_tools() {
        let known = [
            ("coverage", "quality/coverage.md"),
            ("metrics", "perf/metrics.md"),
            ("clippy", "quality/clippy.md"),
            ("fmt", "quality/fmt.md"),
            ("doc", "quality/doc.md"),
            ("deny", "security/deny.md"),
            ("geiger", "security/geiger.md"),
            ("deps", "deps/deps.md"),
            ("msrv", "compat/msrv.md"),
            ("semver", "compat/semver.md"),
            ("udeps", "deps/udeps.md"),
            ("bench", "perf/bench.md"),
            ("bloat", "perf/bloat.md"),
            ("flamegraph", "perf/flamegraph.md"),
            ("binary", "compat/binary.md"),
            ("valgrind_memcheck", "valgrind/memcheck.md"),
            ("valgrind_helgrind", "valgrind/helgrind.md"),
            ("valgrind_drd", "valgrind/drd.md"),
            ("asan", "asan.md"),
        ];
        for (name, expected) in &known {
            assert_eq!(
                builtin_output_path(name),
                Some(*expected),
                "failed for {name}"
            );
        }
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
    fn test_effective_output_path_builtin_fallback() {
        // Built-in tool without explicit config gets default path
        assert_eq!(effective_output_path("clippy", None), "quality/clippy.md");
        assert_eq!(effective_output_path("audit", None), "security/audit.md");
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

    #[test]
    fn test_config_inactive_tool() {
        let toml_str = r#"
[tools.build]
desc = "构建项目"
active = false
input_command = "cargo build"
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert!(!config.tools["build"].active);
    }

    #[test]
    fn test_config_resolve_output_paths_fills_builtins() {
        let toml_str = r#"
[tools.build]
desc = "构建"
active = true
input_command = "cargo build"

[tools.clippy]
desc = "clippy"
active = true
input_command = "cargo clippy"
"#;
        let mut config: Config = toml::from_str(toml_str).unwrap();
        assert!(config.tools["build"].output_path.is_none());
        assert!(config.tools["clippy"].output_path.is_none());

        config.resolve_output_paths();

        assert_eq!(
            config.tools["build"].output_path.as_deref(),
            Some("quality/build.md")
        );
        assert_eq!(
            config.tools["clippy"].output_path.as_deref(),
            Some("quality/clippy.md")
        );
    }

    #[test]
    fn test_config_resolve_output_paths_preserves_existing() {
        let toml_str = r#"
[tools.build]
desc = "构建"
active = true
input_command = "cargo build"
output_path = "my/custom/build.md"
"#;
        let mut config: Config = toml::from_str(toml_str).unwrap();
        config.resolve_output_paths();
        assert_eq!(
            config.tools["build"].output_path.as_deref(),
            Some("my/custom/build.md")
        );
    }

    #[test]
    fn test_config_resolve_output_paths_custom_tool() {
        let toml_str = r#"
[tools.my_custom_tool]
desc = "custom"
active = true
input_command = "echo hello"
"#;
        let mut config: Config = toml::from_str(toml_str).unwrap();
        config.resolve_output_paths();
        assert_eq!(
            config.tools["my_custom_tool"].output_path.as_deref(),
            Some("customs/my_custom_tool.md")
        );
    }

    #[test]
    fn test_config_load_from_file() {
        use std::io::Write;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let mut f = std::fs::File::create(&path).unwrap();
        write!(
            f,
            r#"
schema_version = "1"
[tools.build]
desc = "build"
active = true
input_command = "cargo build"
"#
        )
        .unwrap();

        let config = Config::load(&path).unwrap();
        assert_eq!(config.schema_version.as_deref(), Some("1"));
        assert!(config.tools.contains_key("build"));
    }

    #[test]
    fn test_config_load_missing_file_errors() {
        let result = Config::load(std::path::Path::new("/nonexistent/path/config.toml"));
        assert!(result.is_err());
    }

    #[test]
    fn test_config_load_invalid_toml_errors() {
        use std::io::Write;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let mut f = std::fs::File::create(&path).unwrap();
        write!(f, "this is not valid toml {{{{").unwrap();

        let result = Config::load(&path);
        assert!(result.is_err());
    }
}
