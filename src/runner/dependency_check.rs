use anyhow::Result;
use dialoguer::Confirm;
use std::io::IsTerminal;
use which::which;

pub struct ToolDep {
    pub binary: &'static str,
    /// `cargo install <pkg>` or `rustup component add <comp>` hint.
    pub cargo_install: Option<&'static str>,
    /// Platform-specific system package manager hints (shown when cargo_install is None
    /// or as supplementary information).
    pub system_install: Option<SystemInstallHint>,
}

/// Platform-specific installation hints for tools that may need OS-level packages.
pub struct SystemInstallHint {
    pub linux: Option<&'static str>,
    pub macos: Option<&'static str>,
    pub windows: Option<&'static str>,
}

impl SystemInstallHint {
    /// Return the hint appropriate for the current OS, if any.
    pub fn for_current_os(&self) -> Option<&'static str> {
        #[cfg(target_os = "linux")]
        return self.linux;
        #[cfg(target_os = "macos")]
        return self.macos;
        #[cfg(target_os = "windows")]
        return self.windows;
        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        return None;
    }
}

pub fn get_tool_dep(tool_name: &str) -> Option<ToolDep> {
    match tool_name {
        "build" | "test" | "bench" | "doc" | "deps" | "binary" | "asan" => Some(ToolDep {
            binary: "cargo",
            cargo_install: None,
            system_install: None,
        }),
        "coverage" => Some(ToolDep {
            binary: "cargo-llvm-cov",
            cargo_install: Some("cargo-llvm-cov"),
            // llvm-cov may also need system LLVM on some distros
            system_install: Some(SystemInstallHint {
                linux: Some("sudo apt-get install llvm  # 若 cargo-llvm-cov 安装失败时"),
                macos: Some("brew install llvm  # 若 cargo-llvm-cov 安装失败时"),
                windows: None,
            }),
        }),
        "clippy" => Some(ToolDep {
            binary: "cargo-clippy",
            cargo_install: Some("rustup component add clippy"),
            system_install: None,
        }),
        "fmt" => Some(ToolDep {
            binary: "cargo-fmt",
            cargo_install: Some("rustup component add rustfmt"),
            system_install: None,
        }),
        "audit" => Some(ToolDep {
            binary: "cargo-audit",
            cargo_install: Some("cargo-audit"),
            system_install: None,
        }),
        "deny" => Some(ToolDep {
            binary: "cargo-deny",
            cargo_install: Some("cargo-deny"),
            system_install: None,
        }),
        "geiger" | "metrics" => Some(ToolDep {
            binary: "cargo-geiger",
            cargo_install: Some("cargo-geiger"),
            system_install: None,
        }),
        "msrv" => Some(ToolDep {
            binary: "cargo-msrv",
            cargo_install: Some("cargo-msrv"),
            system_install: None,
        }),
        "semver" => Some(ToolDep {
            binary: "cargo-semver-checks",
            cargo_install: Some("cargo-semver-checks"),
            system_install: None,
        }),
        "udeps" => Some(ToolDep {
            binary: "cargo-udeps",
            cargo_install: Some("cargo-udeps"),
            system_install: None,
        }),
        "bloat" => Some(ToolDep {
            binary: "cargo-bloat",
            cargo_install: Some("cargo-bloat"),
            system_install: None,
        }),
        "flamegraph" => Some(ToolDep {
            binary: "cargo-flamegraph",
            cargo_install: Some("flamegraph"),
            // flamegraph relies on perf (Linux) or DTrace (macOS)
            system_install: Some(SystemInstallHint {
                linux: Some("sudo apt-get install linux-perf  # Linux: 需要 perf 工具"),
                macos: Some("# macOS: 需要 DTrace（系统自带），可能需要关闭 SIP"),
                windows: Some("# Windows: 暂不支持 cargo-flamegraph"),
            }),
        }),
        "valgrind_memcheck" => Some(ToolDep {
            binary: "cargo-valgrind",
            cargo_install: Some("cargo-valgrind"),
            system_install: Some(SystemInstallHint {
                linux: Some("sudo apt-get install valgrind  # cargo-valgrind 需要系统 valgrind"),
                macos: Some("brew install valgrind  # 可用性取决于当前 macOS 版本"),
                windows: Some("# Windows: valgrind/cargo-valgrind 通常不可用"),
            }),
        }),
        "valgrind_helgrind" | "valgrind_drd" => Some(ToolDep {
            binary: "valgrind",
            cargo_install: None,
            system_install: Some(SystemInstallHint {
                linux: Some("sudo apt-get install valgrind"),
                macos: Some("brew install valgrind  # 可用性取决于当前 macOS 版本"),
                windows: Some("# Windows: valgrind 通常不可用"),
            }),
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
        None => {
            // No cargo install path — show system hint if available and bail
            if let Some(sys) = &dep.system_install {
                if let Some(hint) = sys.for_current_os() {
                    eprintln!("  [info] 系统安装提示: {hint}");
                }
            }
            return Ok(false);
        }
    };

    // Print supplementary system-level hint when available
    if let Some(sys) = &dep.system_install {
        if let Some(hint) = sys.for_current_os() {
            eprintln!("  [info] 系统依赖提示: {hint}");
        }
    }

    let prompt = format!(
        "工具 '{tool_name}'（二进制：{}）未找到。是否通过 `{install_hint}` 自动安装？",
        dep.binary
    );

    let confirm = Confirm::new().with_prompt(&prompt).default(true).interact();

    match confirm {
        Ok(true) => {
            println!("正在安装 {tool_name}...");
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
        for tool in &["build", "test", "bench", "doc", "deps", "binary", "asan"] {
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
    fn test_valgrind_memcheck_uses_cargo_valgrind() {
        let dep = get_tool_dep("valgrind_memcheck").unwrap();
        assert_eq!(dep.binary, "cargo-valgrind");
        assert_eq!(dep.cargo_install, Some("cargo-valgrind"));
        assert!(dep.system_install.is_some());
    }

    #[test]
    fn test_valgrind_thread_tools_use_system_valgrind() {
        for tool in &["valgrind_helgrind", "valgrind_drd"] {
            let dep = get_tool_dep(tool).unwrap_or_else(|| panic!("expected dep for {tool}"));
            assert_eq!(dep.binary, "valgrind");
            assert!(dep.cargo_install.is_none());
            assert!(dep.system_install.is_some());
        }
    }

    #[test]
    fn test_flamegraph_has_system_install_hints() {
        let dep = get_tool_dep("flamegraph").unwrap();
        let hint = dep.system_install.unwrap();
        assert!(hint.linux.is_some(), "flamegraph should have a Linux hint");
        assert!(hint.macos.is_some(), "flamegraph should have a macOS hint");
    }

    #[test]
    fn test_coverage_has_system_install_hint() {
        let dep = get_tool_dep("coverage").unwrap();
        let hint = dep.system_install.unwrap();
        assert!(hint.linux.is_some(), "coverage should have a Linux hint");
        assert!(hint.macos.is_some(), "coverage should have a macOS hint");
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
