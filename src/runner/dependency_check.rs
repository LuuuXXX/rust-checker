use anyhow::Result;
use dialoguer::Confirm;
use std::io::IsTerminal;
use which::which;

pub struct ToolDep {
    pub binary: &'static str,
    pub cargo_install: Option<&'static str>,
}

pub fn get_tool_dep(tool_name: &str) -> Option<ToolDep> {
    match tool_name {
        "build" | "test" | "bench" | "doc" | "deps" | "binary" => Some(ToolDep {
            binary: "cargo",
            cargo_install: None,
        }),
        "coverage" => Some(ToolDep {
            binary: "cargo-llvm-cov",
            cargo_install: Some("cargo-llvm-cov"),
        }),
        "clippy" => Some(ToolDep {
            binary: "cargo-clippy",
            cargo_install: Some("rustup component add clippy"),
        }),
        "fmt" => Some(ToolDep {
            binary: "cargo-fmt",
            cargo_install: Some("rustup component add rustfmt"),
        }),
        "audit" => Some(ToolDep {
            binary: "cargo-audit",
            cargo_install: Some("cargo-audit"),
        }),
        "deny" => Some(ToolDep {
            binary: "cargo-deny",
            cargo_install: Some("cargo-deny"),
        }),
        "geiger" | "metrics" => Some(ToolDep {
            binary: "cargo-geiger",
            cargo_install: Some("cargo-geiger"),
        }),
        "msrv" => Some(ToolDep {
            binary: "cargo-msrv",
            cargo_install: Some("cargo-msrv"),
        }),
        "semver" => Some(ToolDep {
            binary: "cargo-semver-checks",
            cargo_install: Some("cargo-semver-checks"),
        }),
        "udeps" => Some(ToolDep {
            binary: "cargo-udeps",
            cargo_install: Some("cargo-udeps"),
        }),
        "bloat" => Some(ToolDep {
            binary: "cargo-bloat",
            cargo_install: Some("cargo-bloat"),
        }),
        "flamegraph" => Some(ToolDep {
            binary: "cargo-flamegraph",
            cargo_install: Some("flamegraph"),
        }),
        _ => None,
    }
}

pub fn check_tool_available(tool_name: &str) -> bool {
    let dep = match get_tool_dep(tool_name) {
        Some(d) => d,
        None => return true,
    };
    which(dep.binary).is_ok()
}

pub fn prompt_and_install(tool_name: &str, dep: &ToolDep) -> Result<bool> {
    if !std::io::stdin().is_terminal() {
        eprintln!(
            "  [warn] {} not found (binary: {}), skipping in non-TTY mode",
            tool_name, dep.binary
        );
        return Ok(false);
    }

    let install_hint = match dep.cargo_install {
        Some(cmd) if cmd.starts_with("rustup") => cmd.to_string(),
        Some(pkg) => format!("cargo install {}", pkg),
        None => return Ok(false),
    };

    let prompt = format!(
        "Tool '{}' (binary: {}) not found. Install with `{}`?",
        tool_name, dep.binary, install_hint
    );

    let confirm = Confirm::new().with_prompt(&prompt).default(true).interact();

    match confirm {
        Ok(true) => {
            println!("Installing {}...", tool_name);
            let status = if dep
                .cargo_install
                .map(|c| c.starts_with("rustup"))
                .unwrap_or(false)
            {
                let parts: Vec<&str> = dep.cargo_install.unwrap().split_whitespace().collect();
                std::process::Command::new(parts[0])
                    .args(&parts[1..])
                    .status()?
            } else {
                let pkg = dep.cargo_install.unwrap();
                std::process::Command::new("cargo")
                    .args(["install", pkg])
                    .status()?
            };
            Ok(status.success())
        }
        _ => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_tool_dep_known_tools() {
        // Tools that always use cargo (no special binary needed)
        for tool in &["build", "test", "bench", "doc", "deps", "binary"] {
            let dep = get_tool_dep(tool).unwrap_or_else(|| panic!("expected dep for {tool}"));
            assert_eq!(dep.binary, "cargo");
            assert!(dep.cargo_install.is_none());
        }
    }

    #[test]
    fn test_get_tool_dep_cargo_subcommands() {
        let cases = [
            ("coverage", "cargo-llvm-cov"),
            ("clippy", "cargo-clippy"),
            ("fmt", "cargo-fmt"),
            ("audit", "cargo-audit"),
            ("deny", "cargo-deny"),
            ("geiger", "cargo-geiger"),
            ("metrics", "cargo-geiger"),
            ("msrv", "cargo-msrv"),
            ("semver", "cargo-semver-checks"),
            ("udeps", "cargo-udeps"),
            ("bloat", "cargo-bloat"),
            ("flamegraph", "cargo-flamegraph"),
        ];
        for (tool, expected_binary) in &cases {
            let dep = get_tool_dep(tool).unwrap_or_else(|| panic!("expected dep for {tool}"));
            assert_eq!(dep.binary, *expected_binary, "wrong binary for {tool}");
            assert!(
                dep.cargo_install.is_some(),
                "expected install hint for {tool}"
            );
        }
    }

    #[test]
    fn test_flamegraph_install_hint_uses_correct_crate_name() {
        // The crate on crates.io is `flamegraph`, not `cargo-flamegraph`.
        // The *binary* installed is `cargo-flamegraph`.
        let dep = get_tool_dep("flamegraph").unwrap();
        assert_eq!(dep.binary, "cargo-flamegraph");
        assert_eq!(dep.cargo_install, Some("flamegraph"));
    }

    #[test]
    fn test_get_tool_dep_unknown_tool_returns_none() {
        assert!(get_tool_dep("nonexistent_tool_xyz").is_none());
    }

    #[test]
    fn test_check_tool_available_cargo_always_true() {
        // cargo should always be available in a CI/dev environment
        assert!(check_tool_available("build"));
        assert!(check_tool_available("test"));
    }

    #[test]
    fn test_check_tool_available_unknown_tool_returns_true() {
        // Unknown tools (no dep info) are assumed available
        assert!(check_tool_available("some_unknown_tool_xyz"));
    }
}
