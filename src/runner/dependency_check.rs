use anyhow::Result;
use std::io::IsTerminal;
use which::which;
use dialoguer::Confirm;

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
            cargo_install: Some("cargo-flamegraph"),
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

    let confirm = Confirm::new()
        .with_prompt(&prompt)
        .default(true)
        .interact();

    match confirm {
        Ok(true) => {
            println!("Installing {}...", tool_name);
            let status =
                if dep.cargo_install.map(|c| c.starts_with("rustup")).unwrap_or(false) {
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
