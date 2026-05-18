use crate::cli::{InitArgs, Preset};
use crate::config::{Config, HistoryConfig, RustConfig, ToolConfig, WatchConfig};
use anyhow::Result;
use std::collections::HashMap;

pub fn run(args: InitArgs) -> Result<()> {
    let config_path = crate::config::config_path();
    if config_path.exists() {
        println!(
            "Config already exists at {:?}. Remove it first to reinitialize.",
            config_path
        );
        return Ok(());
    }

    let config = if let Some(preset) = args.preset {
        build_preset_config(preset)
    } else {
        build_preset_config(Preset::Standard)
    };

    crate::config::save_config(&config)?;
    println!("Initialized rust-checker config at {:?}", config_path);
    Ok(())
}

fn build_preset_config(preset: Preset) -> Config {
    let mut tools: HashMap<String, ToolConfig> = HashMap::new();

    // Minimal tools
    tools.insert(
        "build".to_string(),
        ToolConfig {
            desc: Some("Build the project".to_string()),
            active: Some("true".to_string()),
            input_command: Some("cargo build".to_string()),
            output_path: Some(".localcheck/reports/build".to_string()),
            depends_on: None,
        },
    );
    tools.insert(
        "test".to_string(),
        ToolConfig {
            desc: Some("Run tests".to_string()),
            active: Some("true".to_string()),
            input_command: Some("cargo test".to_string()),
            output_path: Some(".localcheck/reports/test".to_string()),
            depends_on: Some(vec!["build".to_string()]),
        },
    );
    tools.insert(
        "clippy".to_string(),
        ToolConfig {
            desc: Some("Run clippy linter".to_string()),
            active: Some("true".to_string()),
            input_command: Some("cargo clippy -- -D warnings".to_string()),
            output_path: Some(".localcheck/reports/clippy".to_string()),
            depends_on: Some(vec!["build".to_string()]),
        },
    );
    tools.insert(
        "fmt".to_string(),
        ToolConfig {
            desc: Some("Check formatting".to_string()),
            active: Some("true".to_string()),
            input_command: Some("cargo fmt --check".to_string()),
            output_path: Some(".localcheck/reports/fmt".to_string()),
            depends_on: None,
        },
    );

    match preset {
        Preset::Minimal => {}
        Preset::Standard | Preset::Full => {
            tools.insert(
                "doc".to_string(),
                ToolConfig {
                    desc: Some("Generate documentation".to_string()),
                    active: Some("true".to_string()),
                    input_command: Some("cargo doc --no-deps".to_string()),
                    output_path: Some(".localcheck/reports/doc".to_string()),
                    depends_on: Some(vec!["build".to_string()]),
                },
            );
            tools.insert(
                "audit".to_string(),
                ToolConfig {
                    desc: Some("Security audit".to_string()),
                    active: Some("true".to_string()),
                    input_command: Some("cargo audit".to_string()),
                    output_path: Some(".localcheck/reports/audit".to_string()),
                    depends_on: None,
                },
            );
        }
    }

    if matches!(preset, Preset::Full) {
        for (name, cmd) in &[
            ("deny", "cargo deny check"),
            ("geiger", "cargo geiger"),
            ("deps", "cargo tree"),
            ("udeps", "cargo +nightly udeps"),
            ("bloat", "cargo bloat"),
            ("msrv", "cargo msrv"),
            ("semver-checks", "cargo semver-checks"),
        ] {
            tools.insert(
                name.to_string(),
                ToolConfig {
                    desc: Some(format!("Run {}", name)),
                    active: Some("true".to_string()),
                    input_command: Some(cmd.to_string()),
                    output_path: Some(format!(".localcheck/reports/{}", name)),
                    depends_on: Some(vec!["build".to_string()]),
                },
            );
        }
    }

    Config {
        schema_version: Some(1),
        rust: Some(RustConfig {
            version: Some("stable".to_string()),
            rustflags: None,
        }),
        tools: Some(tools),
        history: Some(HistoryConfig {
            max_entries: Some(50),
        }),
        watch: Some(WatchConfig {
            debounce_ms: Some(500),
            rules: None,
        }),
    }
}
