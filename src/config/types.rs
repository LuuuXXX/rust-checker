use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    pub schema_version: Option<u32>,
    pub rust: Option<RustConfig>,
    pub tools: Option<HashMap<String, ToolConfig>>,
    pub history: Option<HistoryConfig>,
    pub watch: Option<WatchConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RustConfig {
    pub version: Option<String>,
    pub rustflags: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ToolConfig {
    pub desc: Option<String>,
    pub active: Option<String>,
    pub input_command: Option<String>,
    pub output_path: Option<String>,
    pub depends_on: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HistoryConfig {
    pub max_entries: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WatchConfig {
    pub debounce_ms: Option<u64>,
    pub rules: Option<HashMap<String, WatchRule>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WatchRule {
    pub pattern: Option<String>,
    pub tools: Option<Vec<String>>,
}
