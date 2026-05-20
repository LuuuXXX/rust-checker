use anyhow::{Context, Result};
use std::path::Path;

/// Current schema version.
pub const CURRENT_SCHEMA_VERSION: &str = "2";

/// Upgrade `.rust-checker/config.toml` to the latest schema version.
///
/// Before migrating the file is backed up to `config.toml.bak`.
pub fn upgrade_config(project_dir: &Path) -> Result<()> {
    let config_path = project_dir.join(".rust-checker").join("config.toml");

    if !config_path.exists() {
        anyhow::bail!(
            "配置文件不存在: {}\n请先运行 `rust-checker init` 生成配置文件",
            config_path.display()
        );
    }

    let content = std::fs::read_to_string(&config_path)
        .with_context(|| format!("读取配置文件失败: {}", config_path.display()))?;

    let current_version = detect_version(&content);

    if current_version == CURRENT_SCHEMA_VERSION {
        println!("✅ 配置文件已是最新版本 (schema_version = \"{CURRENT_SCHEMA_VERSION}\")");
        return Ok(());
    }

    // Backup
    let backup_path = config_path.with_extension("toml.bak");
    std::fs::copy(&config_path, &backup_path)
        .with_context(|| format!("备份配置文件失败: {}", backup_path.display()))?;
    println!("📂 已备份原配置至: {}", backup_path.display());

    // Apply migrations
    let new_content = migrate_to_current(&content, &current_version)?;

    std::fs::write(&config_path, &new_content).with_context(|| "写入升级后配置失败")?;

    println!("✅ 配置文件已升级至 schema_version = \"{CURRENT_SCHEMA_VERSION}\"");

    Ok(())
}

/// Detect the `schema_version` string from raw TOML content.
///
/// Returns `"0"` if the field is absent (pre-versioned configs).
pub fn detect_version(content: &str) -> String {
    #[derive(serde::Deserialize)]
    struct VersionOnly {
        schema_version: Option<String>,
    }
    toml::from_str::<VersionOnly>(content)
        .ok()
        .and_then(|v| v.schema_version)
        .unwrap_or_else(|| "0".to_string())
}

fn migrate_to_current(content: &str, from_version: &str) -> Result<String> {
    let mut result = content.to_string();

    // v0 → v1: add schema_version field
    if from_version == "0" {
        result = format!("schema_version = \"1\"\n\n{result}");
    }

    // v1 → v2: add [history] block if absent
    if !result.contains("[history]") {
        result.push_str("\n# 历史记录配置（Phase 3 新增）\n[history]\nmax_entries = 10\n");
    }

    // Bump schema_version to current
    result = replace_schema_version(&result, CURRENT_SCHEMA_VERSION);

    Ok(result)
}

fn replace_schema_version(content: &str, new_version: &str) -> String {
    let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
    let mut found = false;
    for line in &mut lines {
        if line.trim_start().starts_with("schema_version") {
            *line = format!("schema_version = \"{new_version}\"");
            found = true;
            break;
        }
    }
    if !found {
        lines.insert(0, format!("schema_version = \"{new_version}\""));
    }
    let mut result = lines.join("\n");
    // str::lines() strips the trailing newline; restore it so we don't mutate
    // config files that originally ended with a newline.
    if content.ends_with('\n') {
        result.push('\n');
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_config(dir: &Path, content: &str) {
        let lc = dir.join(".rust-checker");
        std::fs::create_dir_all(&lc).unwrap();
        let mut f = std::fs::File::create(lc.join("config.toml")).unwrap();
        write!(f, "{}", content).unwrap();
    }

    #[test]
    fn test_detect_version_with_version() {
        let content = "schema_version = \"1\"\n[tools]\n";
        assert_eq!(detect_version(content), "1");
    }

    #[test]
    fn test_detect_version_missing() {
        let content = "[tools.build]\ndesc = \"build\"\n";
        assert_eq!(detect_version(content), "0");
    }

    #[test]
    fn test_detect_version_single_quotes() {
        // TOML supports single-quoted strings
        let content = "schema_version = '2'\n";
        assert_eq!(detect_version(content), "2");
    }

    #[test]
    fn test_detect_version_no_space_around_equals() {
        // Compact notation is valid TOML
        let content = "schema_version=\"1\"\n";
        assert_eq!(detect_version(content), "1");
    }

    #[test]
    fn test_detect_version_comment_not_matched() {
        // A comment containing schema_version should not be picked up
        let content = "# schema_version = \"old\"\nfoo = true\n";
        assert_eq!(detect_version(content), "0");
    }

    #[test]
    fn test_upgrade_already_current() {
        let dir = tempfile::tempdir().unwrap();
        let content =
            format!("schema_version = \"{CURRENT_SCHEMA_VERSION}\"\n[history]\nmax_entries = 10\n");
        write_config(dir.path(), &content);

        upgrade_config(dir.path()).unwrap();

        let after =
            std::fs::read_to_string(dir.path().join(".rust-checker").join("config.toml")).unwrap();
        // File should be unchanged
        assert!(after.contains(CURRENT_SCHEMA_VERSION));
        // No backup created
        assert!(!dir
            .path()
            .join(".rust-checker")
            .join("config.toml.bak")
            .exists());
    }

    #[test]
    fn test_upgrade_from_v1_creates_backup() {
        let dir = tempfile::tempdir().unwrap();
        write_config(
            dir.path(),
            "schema_version = \"1\"\n\n[tools.build]\ndesc = \"build\"\nactive = true\ninput_command = \"cargo build\"\n",
        );

        upgrade_config(dir.path()).unwrap();

        assert!(dir
            .path()
            .join(".rust-checker")
            .join("config.toml.bak")
            .exists());
    }

    #[test]
    fn test_upgrade_from_v1_updates_version() {
        let dir = tempfile::tempdir().unwrap();
        write_config(
            dir.path(),
            "schema_version = \"1\"\n\n[tools.build]\ndesc = \"build\"\nactive = true\ninput_command = \"cargo build\"\n",
        );

        upgrade_config(dir.path()).unwrap();

        let after =
            std::fs::read_to_string(dir.path().join(".rust-checker").join("config.toml")).unwrap();
        assert!(
            after.contains(&format!("schema_version = \"{CURRENT_SCHEMA_VERSION}\"")),
            "expected v{CURRENT_SCHEMA_VERSION} in:\n{after}"
        );
    }

    #[test]
    fn test_upgrade_no_config_errors() {
        let dir = tempfile::tempdir().unwrap();
        let result = upgrade_config(dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_replace_schema_version() {
        let content = "schema_version = \"1\"\nother = true\n";
        let updated = replace_schema_version(content, "2");
        assert!(updated.contains("schema_version = \"2\""));
        assert!(!updated.contains("schema_version = \"1\""));
    }

    #[test]
    fn test_replace_schema_version_preserves_trailing_newline() {
        let content = "schema_version = \"1\"\nfoo = true\n";
        let updated = replace_schema_version(content, "2");
        assert!(
            updated.ends_with('\n'),
            "trailing newline should be preserved"
        );
    }

    #[test]
    fn test_replace_schema_version_no_trailing_newline() {
        let content = "schema_version = \"1\"";
        let updated = replace_schema_version(content, "2");
        // Original had no trailing newline — result should not add one either.
        assert!(!updated.ends_with('\n'));
    }
}
