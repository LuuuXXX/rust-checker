pub mod types;
pub use types::*;

use anyhow::Result;
use std::path::PathBuf;

pub fn config_path() -> PathBuf {
    PathBuf::from(".localcheck/config.toml")
}

pub fn load_config() -> Result<Config> {
    let path = config_path();
    if !path.exists() {
        return Ok(Config::default());
    }
    let content = std::fs::read_to_string(&path)?;
    let config: Config = toml::from_str(&content)?;
    Ok(config)
}

pub fn save_config(config: &Config) -> Result<()> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = toml::to_string_pretty(config)?;
    std::fs::write(&path, content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_config_round_trip() {
        let mut tools = HashMap::new();
        tools.insert(
            "build".to_string(),
            ToolConfig {
                desc: Some("Build the project".to_string()),
                active: Some("true".to_string()),
                input_command: Some("cargo build".to_string()),
                output_path: None,
                depends_on: None,
            },
        );
        let config = Config {
            schema_version: Some(1),
            rust: Some(RustConfig {
                version: Some("stable".to_string()),
                rustflags: None,
            }),
            tools: Some(tools),
            history: Some(HistoryConfig {
                max_entries: Some(50),
            }),
            watch: None,
        };
        let serialized = toml::to_string_pretty(&config).unwrap();
        let deserialized: Config = toml::from_str(&serialized).unwrap();
        assert_eq!(deserialized.schema_version, Some(1));
        assert_eq!(
            deserialized.rust.unwrap().version,
            Some("stable".to_string())
        );
    }

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.schema_version.is_none());
        assert!(config.tools.is_none());
    }
}
