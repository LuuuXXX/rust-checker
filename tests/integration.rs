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
    assert!(dir.path().join(".localcheck").join("config.toml").exists());
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
        std::fs::read_to_string(dir.path().join(".localcheck").join("config.toml")).unwrap();
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
        std::fs::read_to_string(dir.path().join(".localcheck").join("config.toml")).unwrap();
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
    let config_path = dir.path().join(".localcheck").join("config.toml");
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
    let config_path = dir.path().join(".localcheck").join("config.toml");
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
fn test_run_empty_tools_config_errors() {
    let dir = temp_dir();
    let localcheck = dir.path().join(".localcheck");
    std::fs::create_dir_all(&localcheck).unwrap();
    // Config with no tools
    std::fs::write(
        localcheck.join("config.toml"),
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
    let localcheck = dir.path().join(".localcheck");
    std::fs::create_dir_all(&localcheck).unwrap();

    // Config with one inactive tool (so it skips immediately without running cargo)
    std::fs::write(
        localcheck.join("config.toml"),
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
    let json_path = localcheck.join("reports").join("ci_result.json");
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
    let localcheck = dir.path().join(".localcheck");
    std::fs::create_dir_all(&localcheck).unwrap();

    // Config with one inactive tool
    std::fs::write(
        localcheck.join("config.toml"),
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

    let summary_path = localcheck.join("reports").join("summary.md");
    assert!(summary_path.exists(), "summary.md not found");
    let content = std::fs::read_to_string(&summary_path).unwrap();
    assert!(content.contains("build"));
}
