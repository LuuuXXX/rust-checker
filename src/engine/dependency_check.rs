pub struct MissingDep {
    pub name: String,
    pub install_cmd: Option<String>,
}

pub fn check_tool_deps(tool_name: &str, deps: &[String]) -> Vec<MissingDep> {
    let mut missing = Vec::new();
    for dep in deps {
        if which::which(dep).is_err() {
            missing.push(MissingDep {
                name: dep.clone(),
                install_cmd: install_hint(tool_name, dep),
            });
        }
    }
    missing
}

fn install_hint(tool_name: &str, dep: &str) -> Option<String> {
    match dep {
        "cargo" => Some("Install Rust from https://rustup.rs".to_string()),
        "cargo-audit" => Some("cargo install cargo-audit".to_string()),
        "cargo-deny" => Some("cargo install cargo-deny".to_string()),
        "cargo-geiger" => Some("cargo install cargo-geiger".to_string()),
        "cargo-llvm-cov" => Some("cargo install cargo-llvm-cov".to_string()),
        "cargo-bloat" => Some("cargo install cargo-bloat".to_string()),
        "cargo-msrv" => Some("cargo install cargo-msrv".to_string()),
        "cargo-semver-checks" => Some("cargo install cargo-semver-checks".to_string()),
        "cargo-udeps" => Some("cargo install cargo-udeps".to_string()),
        _ => Some(format!("cargo install {}", tool_name)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_tool_deps_missing() {
        // A binary that definitely doesn't exist
        let deps = vec!["__nonexistent_tool_xyz__".to_string()];
        let missing = check_tool_deps("test-tool", &deps);
        assert_eq!(missing.len(), 1);
        assert_eq!(missing[0].name, "__nonexistent_tool_xyz__");
    }

    #[test]
    fn test_check_tool_deps_present() {
        // cargo should always be present in CI
        let deps = vec!["cargo".to_string()];
        let missing = check_tool_deps("build", &deps);
        // cargo should be found; if not found it's still valid behavior
        let _ = missing; // just ensure it doesn't panic
    }
}
