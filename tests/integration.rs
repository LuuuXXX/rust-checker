//! Integration tests for the `rust-checker` CLI binary.
//!
//! These tests invoke the built binary (`rust-checker`) directly via
//! `std::process::Command` to validate end-to-end behaviour.

use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

/// Return the path to the compiled `rust-checker` binary.
fn bin() -> PathBuf {
    let mut path = std::env::current_exe()
        .expect("current_exe")
        .canonicalize()
        .expect("canonicalize");
    // Strip the test-binary filename
    path.pop();
    // If inside a `deps/` directory, go up one more level
    if path.ends_with("deps") {
        path.pop();
    }
    let name = if cfg!(windows) {
        "rust-checker.exe"
    } else {
        "rust-checker"
    };
    path.push(name);
    path
}

/// Create a `TempDir` and return the directory, asserting creation succeeded.
fn temp_dir() -> TempDir {
    tempfile::tempdir().expect("tempdir")
}

// ---------------------------------------------------------------------------
// --help / --version smoke tests
// ---------------------------------------------------------------------------

#[test]
fn test_help_flag() {
    let out = Command::new(bin())
        .arg("--help")
        .output()
        .expect("run binary");
    assert!(out.status.success(), "exit {:?}", out.status);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("rust-checker") || stdout.contains("Usage"),
        "expected help text, got: {stdout}"
    );
}

#[test]
fn test_version_flag() {
    let out = Command::new(bin())
        .arg("--version")
        .output()
        .expect("run binary");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("rust-checker"), "expected version string");
}

// ---------------------------------------------------------------------------
// `rust-checker init` integration tests
// ---------------------------------------------------------------------------

#[test]
fn test_init_creates_config_file() {
    let dir = temp_dir();
    let out = Command::new(bin())
        .args(["init", "--dir"])
        .arg(dir.path())
        .args(["--preset", "minimal"])
        .output()
        .expect("run init");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(dir
        .path()
        .join(".rust-checker")
        .join("config.toml")
        .exists());
}

#[test]
fn test_init_minimal_preset_content() {
    let dir = temp_dir();
    Command::new(bin())
        .args(["init", "--dir"])
        .arg(dir.path())
        .args(["--preset", "minimal"])
        .output()
        .expect("run init");

    let content =
        std::fs::read_to_string(dir.path().join(".rust-checker").join("config.toml")).unwrap();
    assert!(content.contains("build"));
    assert!(content.contains("test"));
    assert!(content.contains("clippy"));
    assert!(content.contains("fmt"));
}

#[test]
fn test_init_full_preset_has_all_tools() {
    let dir = temp_dir();
    Command::new(bin())
        .args(["init", "--dir"])
        .arg(dir.path())
        .args(["--preset", "full"])
        .output()
        .expect("run init");

    let content =
        std::fs::read_to_string(dir.path().join(".rust-checker").join("config.toml")).unwrap();
    // Spot-check several tools from all categories
    for tool in &[
        "build", "test", "clippy", "audit", "geiger", "msrv", "bloat", "bench",
    ] {
        assert!(content.contains(tool), "full preset missing tool: {tool}");
    }
}

#[test]
fn test_init_no_overwrite_without_force() {
    let dir = temp_dir();
    // First init
    Command::new(bin())
        .args(["init", "--dir"])
        .arg(dir.path())
        .args(["--preset", "minimal"])
        .output()
        .expect("first init");

    // Overwrite with custom marker
    let config_path = dir.path().join(".rust-checker").join("config.toml");
    std::fs::write(&config_path, "# sentinel_marker\n").unwrap();

    // Second init without --force should NOT overwrite
    Command::new(bin())
        .args(["init", "--dir"])
        .arg(dir.path())
        .args(["--preset", "full"])
        .output()
        .expect("second init");

    let content = std::fs::read_to_string(&config_path).unwrap();
    assert!(
        content.contains("sentinel_marker"),
        "config was overwritten without --force"
    );
}

#[test]
fn test_init_force_flag_overwrites() {
    let dir = temp_dir();
    // First init
    Command::new(bin())
        .args(["init", "--dir"])
        .arg(dir.path())
        .args(["--preset", "minimal"])
        .output()
        .expect("first init");

    // Overwrite with sentinel
    let config_path = dir.path().join(".rust-checker").join("config.toml");
    std::fs::write(&config_path, "# sentinel_marker\n").unwrap();

    // Second init WITH --force should overwrite
    Command::new(bin())
        .args(["init", "--dir"])
        .arg(dir.path())
        .args(["--preset", "full", "--force"])
        .output()
        .expect("force init");

    let content = std::fs::read_to_string(&config_path).unwrap();
    assert!(
        !content.contains("sentinel_marker"),
        "config was NOT overwritten with --force"
    );
}

// ---------------------------------------------------------------------------
// `rust-checker run` integration tests
// ---------------------------------------------------------------------------

#[test]
fn test_run_fails_without_config() {
    let dir = temp_dir();
    let out = Command::new(bin())
        .args(["run", "--dir"])
        .arg(dir.path())
        .output()
        .expect("run command");

    // Should fail and print a helpful message
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("config") || stderr.contains("init"),
        "expected helpful error, got: {stderr}"
    );
}

#[test]
fn test_run_unknown_tool_in_only_flag_errors() {
    let dir = temp_dir();
    let rust_checker = dir.path().join(".rust-checker");
    std::fs::create_dir_all(&rust_checker).unwrap();
    // Config with no tools
    std::fs::write(
        rust_checker.join("config.toml"),
        r#"schema_version = "1"
[rust]
version = "1.75.0"
"#,
    )
    .unwrap();

    // run with --only pointing to a non-existent tool
    let out = Command::new(bin())
        .args(["run", "--dir"])
        .arg(dir.path())
        .args(["--only", "nonexistent_tool_xyz"])
        .output()
        .expect("run command");

    assert!(!out.status.success());
}

#[test]
fn test_run_ci_mode_creates_json() {
    let dir = temp_dir();
    let rust_checker = dir.path().join(".rust-checker");
    std::fs::create_dir_all(&rust_checker).unwrap();

    // Config with one inactive tool (so it skips immediately without running cargo)
    std::fs::write(
        rust_checker.join("config.toml"),
        r#"schema_version = "1"

[tools.build]
desc = "build"
active = false
input_command = "cargo build"
"#,
    )
    .unwrap();

    let out = Command::new(bin())
        .args(["run", "--dir"])
        .arg(dir.path())
        .args(["--ci"])
        .output()
        .expect("run command");

    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    // CI mode should produce a ci_result.json
    let json_path = rust_checker.join("reports").join("ci_result.json");
    assert!(json_path.exists(), "ci_result.json not found");

    let json_content = std::fs::read_to_string(&json_path).unwrap();
    let v: serde_json::Value = serde_json::from_str(&json_content).unwrap();
    assert!(v["timestamp"].is_string());
    assert!(v["summary"].is_object());
    assert!(v["tools"].is_array());
}

#[test]
fn test_run_generates_summary_md() {
    let dir = temp_dir();
    let rust_checker = dir.path().join(".rust-checker");
    std::fs::create_dir_all(&rust_checker).unwrap();

    // Config with one inactive tool
    std::fs::write(
        rust_checker.join("config.toml"),
        r#"schema_version = "1"

[tools.build]
desc = "build"
active = false
input_command = "cargo build"
"#,
    )
    .unwrap();

    let out = Command::new(bin())
        .args(["run", "--dir"])
        .arg(dir.path())
        .output()
        .expect("run command");

    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let summary_path = rust_checker.join("reports").join("summary.md");
    assert!(summary_path.exists(), "summary.md not found");
    let content = std::fs::read_to_string(&summary_path).unwrap();
    assert!(content.contains("build"));
}

// ---------------------------------------------------------------------------
// Skipped tool report files are written to disk (linked from summary.md)
// ---------------------------------------------------------------------------

#[test]
fn test_run_inactive_tool_report_file_is_written() {
    let dir = temp_dir();
    let rust_checker = dir.path().join(".rust-checker");
    std::fs::create_dir_all(&rust_checker).unwrap();

    std::fs::write(
        rust_checker.join("config.toml"),
        r#"schema_version = "1"

[tools.build]
desc = "build"
active = false
input_command = "cargo build"
"#,
    )
    .unwrap();

    let out = Command::new(bin())
        .args(["run", "--dir"])
        .arg(dir.path())
        .output()
        .expect("run command");

    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    // The skipped tool report file must exist so summary.md links are valid
    let report_path = rust_checker
        .join("reports")
        .join("quality")
        .join("build.md");
    assert!(
        report_path.exists(),
        "skipped tool report quality/build.md not found"
    );
    let content = std::fs::read_to_string(&report_path).unwrap();
    assert!(
        content.contains("build"),
        "skipped report should mention tool name"
    );
}

// ---------------------------------------------------------------------------
// Phase 3: `rust-checker run` creates history entry
// ---------------------------------------------------------------------------

#[test]
fn test_run_creates_history_entry() {
    let dir = temp_dir();
    let rust_checker = dir.path().join(".rust-checker");
    std::fs::create_dir_all(&rust_checker).unwrap();

    std::fs::write(
        rust_checker.join("config.toml"),
        r#"schema_version = "2"

[history]
max_entries = 5

[tools.build]
desc = "build"
active = false
input_command = "cargo build"
"#,
    )
    .unwrap();

    let out = Command::new(bin())
        .args(["run", "--dir"])
        .arg(dir.path())
        .output()
        .expect("run command");

    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let history_dir = rust_checker.join("history");
    assert!(history_dir.exists(), "history directory was not created");

    let entries: Vec<_> = std::fs::read_dir(&history_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert!(!entries.is_empty(), "no history entries were saved");

    // Each entry directory should contain result.json
    for e in &entries {
        let result_json = e.path().join("result.json");
        assert!(
            result_json.exists(),
            "result.json missing in {:?}",
            e.path()
        );
        let content = std::fs::read_to_string(&result_json).unwrap();
        let v: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert!(v["timestamp"].is_string());
        assert!(v["tools"].is_array());
    }
}

// ---------------------------------------------------------------------------
// Phase 3: `rust-checker diff` command
// ---------------------------------------------------------------------------

#[test]
fn test_diff_no_history_errors() {
    let dir = temp_dir();
    let rust_checker = dir.path().join(".rust-checker");
    std::fs::create_dir_all(&rust_checker).unwrap();
    std::fs::write(rust_checker.join("config.toml"), "schema_version = \"2\"\n").unwrap();

    let out = Command::new(bin())
        .args(["diff", "--dir"])
        .arg(dir.path())
        .output()
        .expect("diff command");

    assert!(!out.status.success(), "diff without history should fail");
}

#[test]
fn test_diff_last_shows_trend() {
    let dir = temp_dir();
    let rust_checker = dir.path().join(".rust-checker");
    let history_dir = rust_checker.join("history");
    std::fs::create_dir_all(&history_dir).unwrap();
    std::fs::write(rust_checker.join("config.toml"), "schema_version = \"2\"\n").unwrap();

    // Create two fake history entries
    for ts in &["20260101-100000", "20260101-110000"] {
        let entry_dir = history_dir.join(ts);
        std::fs::create_dir_all(&entry_dir).unwrap();
        std::fs::write(
            entry_dir.join("result.json"),
            format!(
                r#"{{"timestamp":"{ts}","tools":[{{"tool_name":"build","status":"ok","summary":"built"}}]}}"#
            ),
        )
        .unwrap();
    }

    let out = Command::new(bin())
        .args(["diff", "--dir"])
        .arg(dir.path())
        .args(["--last", "2"])
        .output()
        .expect("diff --last");

    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("趋势"),
        "expected trend output, got: {stdout}"
    );
    assert!(
        stdout.contains("20260101"),
        "expected timestamp in trend output, got: {stdout}"
    );
}

// ---------------------------------------------------------------------------
// Phase 3: `rust-checker upgrade` command
// ---------------------------------------------------------------------------

#[test]
fn test_upgrade_no_config_fails() {
    let dir = temp_dir();
    let out = Command::new(bin())
        .args(["upgrade", "--dir"])
        .arg(dir.path())
        .output()
        .expect("upgrade command");
    assert!(!out.status.success(), "upgrade without config should fail");
}

#[test]
fn test_upgrade_migrates_v1_config() {
    let dir = temp_dir();
    let rust_checker = dir.path().join(".rust-checker");
    std::fs::create_dir_all(&rust_checker).unwrap();
    std::fs::write(
        rust_checker.join("config.toml"),
        "schema_version = \"1\"\n\n[tools.build]\ndesc = \"build\"\nactive = true\ninput_command = \"cargo build\"\n",
    )
    .unwrap();

    let out = Command::new(bin())
        .args(["upgrade", "--dir"])
        .arg(dir.path())
        .output()
        .expect("upgrade command");

    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let content = std::fs::read_to_string(rust_checker.join("config.toml")).unwrap();
    assert!(
        content.contains("schema_version = \"2\""),
        "schema_version not updated"
    );
    // Backup should exist
    assert!(
        rust_checker.join("config.toml.bak").exists(),
        "backup not created"
    );
}

// ---------------------------------------------------------------------------
// Phase 3: `rust-checker plugin list` command
// ---------------------------------------------------------------------------

#[test]
fn test_plugin_list_empty() {
    let dir = temp_dir();
    let out = Command::new(bin())
        .args(["plugin", "list", "--dir"])
        .arg(dir.path())
        .output()
        .expect("plugin list");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("没有") || stdout.contains("安装"));
}

// ---------------------------------------------------------------------------
// `rust-checker run --crate` reports go to workspace root .rust-checker/
// ---------------------------------------------------------------------------

#[test]
fn test_run_crate_mode_reports_go_to_workspace_root() {
    let dir = temp_dir();
    let workspace_root = dir.path();
    let rust_checker = workspace_root.join(".rust-checker");
    std::fs::create_dir_all(&rust_checker).unwrap();

    // Create a fake workspace with one member crate
    let crate_dir = workspace_root.join("crates").join("my-lib");
    std::fs::create_dir_all(&crate_dir).unwrap();
    std::fs::write(
        crate_dir.join("Cargo.toml"),
        "[package]\nname = \"my-lib\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    std::fs::write(
        workspace_root.join("Cargo.toml"),
        "[workspace]\nmembers = [\"crates/*\"]\n",
    )
    .unwrap();

    // Config at workspace root
    std::fs::write(
        rust_checker.join("config.toml"),
        r#"schema_version = "1"

[tools.build]
desc = "build"
active = false
input_command = "cargo build"
"#,
    )
    .unwrap();

    let out = Command::new(bin())
        .args(["run", "--dir"])
        .arg(workspace_root)
        .args(["--crate", "my-lib"])
        .output()
        .expect("run --crate command");

    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    // Reports must be written to the WORKSPACE root .rust-checker/reports/, not to
    // crates/my-lib/.rust-checker/reports/
    let report_path = rust_checker.join("reports").join("summary.md");
    assert!(
        report_path.exists(),
        "summary.md should be in workspace root .rust-checker/, not in crate subdir"
    );

    // Crate subdir must NOT have a stray .rust-checker/ directory
    assert!(
        !crate_dir.join(".rust-checker").exists(),
        "stray .rust-checker/ found in crate directory"
    );
}

#[test]
fn test_plugin_remove_nonexistent_is_ok() {
    let dir = temp_dir();
    let out = Command::new(bin())
        .args(["plugin", "remove", "nonexistent-plugin", "--dir"])
        .arg(dir.path())
        .output()
        .expect("plugin remove");
    assert!(out.status.success());
}

// ---------------------------------------------------------------------------
// `rust-checker diff` integration tests (extended)
// ---------------------------------------------------------------------------

#[test]
fn test_diff_latest_with_two_entries() {
    let dir = temp_dir();
    let rust_checker = dir.path().join(".rust-checker");
    let history_dir = rust_checker.join("history");
    std::fs::create_dir_all(&history_dir).unwrap();
    std::fs::write(rust_checker.join("config.toml"), "schema_version = \"2\"\n").unwrap();

    // Create two fake history entries with different statuses
    let entry_a = history_dir.join("20260101-100000");
    let entry_b = history_dir.join("20260101-110000");
    std::fs::create_dir_all(&entry_a).unwrap();
    std::fs::create_dir_all(&entry_b).unwrap();
    std::fs::write(
        entry_a.join("result.json"),
        r#"{"timestamp":"20260101-100000","tools":[{"tool_name":"build","status":"ok","summary":"built"},{"tool_name":"test","status":"ok","summary":"10 passed"}]}"#,
    )
    .unwrap();
    std::fs::write(
        entry_b.join("result.json"),
        r#"{"timestamp":"20260101-110000","tools":[{"tool_name":"build","status":"ok","summary":"built"},{"tool_name":"test","status":"error","summary":"2 failed"}]}"#,
    )
    .unwrap();

    let out = Command::new(bin())
        .args(["diff", "--dir"])
        .arg(dir.path())
        .output()
        .expect("diff command");

    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    // Diff output must identify the regressing tool and both timestamps.
    assert!(
        stdout.contains("test"),
        "diff must mention the regressing tool 'test': {stdout}"
    );
    assert!(
        stdout.contains("20260101"),
        "diff must include the timestamp: {stdout}"
    );
    // The regression: test went from ok → error; look for an indicator of degradation.
    assert!(
        stdout.contains("error") || stdout.contains("↓") || stdout.contains("劣化"),
        "diff must indicate a regression: {stdout}"
    );
}

#[test]
fn test_diff_range_no_match_errors() {
    let dir = temp_dir();
    let rust_checker = dir.path().join(".rust-checker");
    let history_dir = rust_checker.join("history");
    std::fs::create_dir_all(&history_dir).unwrap();
    std::fs::write(rust_checker.join("config.toml"), "schema_version = \"2\"\n").unwrap();

    // One entry outside the queried range
    let entry = history_dir.join("20260601-120000");
    std::fs::create_dir_all(&entry).unwrap();
    std::fs::write(
        entry.join("result.json"),
        r#"{"timestamp":"20260601-120000","tools":[]}"#,
    )
    .unwrap();

    let out = Command::new(bin())
        .args(["diff", "--dir"])
        .arg(dir.path())
        .args(["--from", "20260101", "--to", "20260131"])
        .output()
        .expect("diff --from --to");

    assert!(
        !out.status.success(),
        "diff with no matching range should fail"
    );
}

#[test]
fn test_diff_last_single_entry_shows_trend() {
    let dir = temp_dir();
    let rust_checker = dir.path().join(".rust-checker");
    let history_dir = rust_checker.join("history");
    std::fs::create_dir_all(&history_dir).unwrap();
    std::fs::write(rust_checker.join("config.toml"), "schema_version = \"2\"\n").unwrap();

    // Only one entry
    let entry = history_dir.join("20260101-100000");
    std::fs::create_dir_all(&entry).unwrap();
    std::fs::write(
        entry.join("result.json"),
        r#"{"timestamp":"20260101-100000","tools":[{"tool_name":"build","status":"ok","summary":"built"}]}"#,
    )
    .unwrap();

    let out = Command::new(bin())
        .args(["diff", "--dir"])
        .arg(dir.path())
        .args(["--last", "5"])
        .output()
        .expect("diff --last 5");

    // With only one entry, --last should still succeed (shows trend of 1)
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

// ---------------------------------------------------------------------------
// `rust-checker run --set-cmd` integration tests
// ---------------------------------------------------------------------------

#[test]
fn test_run_set_cmd_overrides_inactive_tool() {
    let dir = temp_dir();
    let rust_checker = dir.path().join(".rust-checker");
    std::fs::create_dir_all(&rust_checker).unwrap();

    std::fs::write(
        rust_checker.join("config.toml"),
        r#"schema_version = "2"

[tools.build]
desc = "build"
active = false
input_command = "cargo build"
"#,
    )
    .unwrap();

    // Override inactive tool's command — should still succeed (tool is inactive/skipped)
    let out = Command::new(bin())
        .args(["run", "--dir"])
        .arg(dir.path())
        .args(["--set-cmd", "build=cargo build --release"])
        .output()
        .expect("run --set-cmd");

    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn test_run_set_cmd_unknown_tool_errors() {
    let dir = temp_dir();
    let rust_checker = dir.path().join(".rust-checker");
    std::fs::create_dir_all(&rust_checker).unwrap();

    std::fs::write(
        rust_checker.join("config.toml"),
        r#"schema_version = "2"

[tools.build]
desc = "build"
active = false
input_command = "cargo build"
"#,
    )
    .unwrap();

    let out = Command::new(bin())
        .args(["run", "--dir"])
        .arg(dir.path())
        .args(["--set-cmd", "nonexistent=cargo build"])
        .output()
        .expect("run with unknown tool");

    assert!(
        !out.status.success(),
        "expected failure for unknown tool in --set-cmd"
    );
}

#[test]
fn test_run_set_cmd_bad_format_errors() {
    let dir = temp_dir();
    let rust_checker = dir.path().join(".rust-checker");
    std::fs::create_dir_all(&rust_checker).unwrap();

    std::fs::write(rust_checker.join("config.toml"), "schema_version = \"2\"\n").unwrap();

    let out = Command::new(bin())
        .args(["run", "--dir"])
        .arg(dir.path())
        .args(["--set-cmd", "no-equals-sign"])
        .output()
        .expect("run with bad set-cmd format");

    assert!(
        !out.status.success(),
        "expected failure for malformed --set-cmd"
    );
}

// ---------------------------------------------------------------------------
// Output format validation: summary.md deep structure
// ---------------------------------------------------------------------------

/// summary.md must contain a heading, a markdown table, tool names, and stat counts.
#[test]
fn test_run_summary_md_structure() {
    let dir = temp_dir();
    let rust_checker = dir.path().join(".rust-checker");
    std::fs::create_dir_all(&rust_checker).unwrap();

    std::fs::write(
        rust_checker.join("config.toml"),
        r#"schema_version = "1"

[tools.clippy]
desc = "clippy"
active = false
input_command = "cargo clippy"

[tools.fmt]
desc = "fmt"
active = false
input_command = "cargo fmt --check"
"#,
    )
    .unwrap();

    let out = Command::new(bin())
        .args(["run", "--dir"])
        .arg(dir.path())
        .output()
        .expect("run command");

    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let content = std::fs::read_to_string(rust_checker.join("reports").join("summary.md")).unwrap();

    // Must have a top-level Markdown heading
    assert!(content.contains("# "), "summary.md missing heading");
    // Must have a Markdown table (pipe characters)
    assert!(content.contains("| "), "summary.md missing table");
    assert!(
        content.contains("|--"),
        "summary.md missing table separator"
    );
    // Must mention tool names
    assert!(
        content.contains("clippy"),
        "summary.md missing tool 'clippy'"
    );
    assert!(content.contains("fmt"), "summary.md missing tool 'fmt'");
    // Statistics section must report two skipped tools
    assert!(
        content.contains("跳过: 2"),
        "summary.md missing expected skip count (got: {content})"
    );
}

// ---------------------------------------------------------------------------
// Output format validation: ci_result.json deep field checks
// ---------------------------------------------------------------------------

/// ci_result.json must contain all summary sub-fields and per-tool fields.
#[test]
fn test_run_ci_json_tool_fields_present() {
    let dir = temp_dir();
    let rust_checker = dir.path().join(".rust-checker");
    std::fs::create_dir_all(&rust_checker).unwrap();

    std::fs::write(
        rust_checker.join("config.toml"),
        r#"schema_version = "1"

[tools.build]
desc = "build"
active = false
input_command = "cargo build"
"#,
    )
    .unwrap();

    let out = Command::new(bin())
        .args(["run", "--dir"])
        .arg(dir.path())
        .args(["--ci"])
        .output()
        .expect("run command");

    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let json_path = rust_checker.join("reports").join("ci_result.json");
    let content = std::fs::read_to_string(&json_path).unwrap();
    let v: serde_json::Value = serde_json::from_str(&content).unwrap();

    // summary sub-fields
    let summary = &v["summary"];
    assert!(summary["total"].is_number(), "summary.total missing");
    assert!(summary["ok"].is_number(), "summary.ok missing");
    assert!(summary["warn"].is_number(), "summary.warn missing");
    assert!(summary["error"].is_number(), "summary.error missing");
    assert!(summary["skipped"].is_number(), "summary.skipped missing");

    // counts must match: one inactive tool → 1 skipped
    assert_eq!(summary["total"], 1, "expected total=1");
    assert_eq!(summary["skipped"], 1, "expected skipped=1");
    assert_eq!(summary["ok"], 0, "expected ok=0");

    // per-tool entry fields
    let tools = v["tools"].as_array().unwrap();
    assert!(!tools.is_empty(), "tools array must not be empty");
    let tool = &tools[0];
    assert!(tool["tool"].is_string(), "tool.tool missing");
    assert!(tool["status"].is_string(), "tool.status missing");
    assert!(tool["summary"].is_string(), "tool.summary missing");
    assert!(tool["output_path"].is_string(), "tool.output_path missing");

    // status value must be one of the known strings
    let status = tool["status"].as_str().unwrap();
    assert!(
        ["ok", "warn", "error", "skipped"].contains(&status),
        "unexpected status value: {status}"
    );
}

// ---------------------------------------------------------------------------
// Output format validation: --format json triggers ci_result.json (no --ci)
// ---------------------------------------------------------------------------

#[test]
fn test_run_format_json_creates_ci_result() {
    let dir = temp_dir();
    let rust_checker = dir.path().join(".rust-checker");
    std::fs::create_dir_all(&rust_checker).unwrap();

    std::fs::write(
        rust_checker.join("config.toml"),
        r#"schema_version = "1"

[tools.build]
desc = "build"
active = false
input_command = "cargo build"
"#,
    )
    .unwrap();

    let out = Command::new(bin())
        .args(["run", "--dir"])
        .arg(dir.path())
        .args(["--format", "json"])
        .output()
        .expect("run command");

    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    // --format json (without --ci) should also produce ci_result.json
    let json_path = rust_checker.join("reports").join("ci_result.json");
    assert!(
        json_path.exists(),
        "ci_result.json not created with --format json"
    );

    let content = std::fs::read_to_string(&json_path).unwrap();
    let v: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert!(v["timestamp"].is_string(), "timestamp must be string");
    assert!(v["summary"].is_object(), "summary must be object");
    assert!(v["tools"].is_array(), "tools must be array");
}

// ---------------------------------------------------------------------------
// Output format validation: --format html
// ---------------------------------------------------------------------------

#[test]
fn test_run_format_html_creates_summary_html() {
    let dir = temp_dir();
    let rust_checker = dir.path().join(".rust-checker");
    std::fs::create_dir_all(&rust_checker).unwrap();

    std::fs::write(
        rust_checker.join("config.toml"),
        r#"schema_version = "1"

[tools.build]
desc = "build"
active = false
input_command = "cargo build"
"#,
    )
    .unwrap();

    let out = Command::new(bin())
        .args(["run", "--dir"])
        .arg(dir.path())
        .args(["--format", "html"])
        .output()
        .expect("run command");

    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let summary_html = rust_checker.join("reports").join("summary.html");
    assert!(summary_html.exists(), "summary.html not found");

    let content = std::fs::read_to_string(&summary_html).unwrap();
    assert!(
        content.contains("<!DOCTYPE html>"),
        "summary.html missing DOCTYPE"
    );
    assert!(
        content.contains("stat-bar"),
        "summary.html missing stat-bar section"
    );
    assert!(content.contains("grid"), "summary.html missing grid layout");
    assert!(content.contains("build"), "summary.html missing tool name");
}

#[test]
fn test_run_format_html_creates_tool_html_reports() {
    let dir = temp_dir();
    let rust_checker = dir.path().join(".rust-checker");
    std::fs::create_dir_all(&rust_checker).unwrap();

    std::fs::write(
        rust_checker.join("config.toml"),
        r#"schema_version = "1"

[tools.fmt]
desc = "fmt"
active = false
input_command = "cargo fmt --check"
"#,
    )
    .unwrap();

    let out = Command::new(bin())
        .args(["run", "--dir"])
        .arg(dir.path())
        .args(["--format", "html"])
        .output()
        .expect("run command");

    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    // HTML tool report should exist alongside the markdown one
    let html_report = rust_checker
        .join("reports")
        .join("quality")
        .join("fmt.html");
    assert!(html_report.exists(), "quality/fmt.html not found");

    let content = std::fs::read_to_string(&html_report).unwrap();
    assert!(
        content.contains("<!DOCTYPE html>"),
        "tool HTML report missing DOCTYPE"
    );
    assert!(
        content.contains("fmt"),
        "tool HTML report missing tool name"
    );
    assert!(
        content.contains("badge"),
        "tool HTML report missing status badge"
    );
}

#[test]
fn test_run_format_html_summary_counts() {
    let dir = temp_dir();
    let rust_checker = dir.path().join(".rust-checker");
    std::fs::create_dir_all(&rust_checker).unwrap();

    std::fs::write(
        rust_checker.join("config.toml"),
        r#"schema_version = "1"

[tools.build]
desc = "build"
active = false
input_command = "cargo build"

[tools.test]
desc = "test"
active = false
input_command = "cargo test"
"#,
    )
    .unwrap();

    let out = Command::new(bin())
        .args(["run", "--dir"])
        .arg(dir.path())
        .args(["--format", "html"])
        .output()
        .expect("run command");

    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let content =
        std::fs::read_to_string(rust_checker.join("reports").join("summary.html")).unwrap();
    // The HTML stat bar must contain the skipped count (2 inactive tools → 2 skipped)
    assert!(content.contains(">2<"), "summary.html missing count '2'");
    // Must contain status labels (HTML always uses Chinese labels from ToolStatus::label())
    assert!(
        content.contains("通过"),
        "summary.html missing ok label '通过'"
    );
    assert!(
        content.contains("跳过"),
        "summary.html missing skipped label '跳过'"
    );
}
