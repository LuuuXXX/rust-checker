use anyhow::Result;
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct WorkspaceInfo {
    pub root: PathBuf,
    pub members: Vec<WorkspaceMember>,
}

#[derive(Debug, Clone)]
pub struct WorkspaceMember {
    pub name: String,
    pub path: PathBuf,
}

#[derive(Deserialize)]
struct CargoToml {
    workspace: Option<WorkspaceSection>,
}

#[derive(Deserialize)]
struct WorkspaceSection {
    members: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct PackageManifest {
    package: Option<PackageSection>,
}

#[derive(Deserialize)]
struct PackageSection {
    name: Option<String>,
}

fn get_crate_name(dir: &Path) -> Option<String> {
    let content = std::fs::read_to_string(dir.join("Cargo.toml")).ok()?;
    let manifest: PackageManifest = toml::from_str(&content).ok()?;
    manifest.package?.name
}

/// Detect a Cargo workspace at `project_dir`.
/// Returns `None` if there is no `[workspace]` section in `Cargo.toml`.
pub fn detect_workspace(project_dir: &Path) -> Option<WorkspaceInfo> {
    let cargo_toml_path = project_dir.join("Cargo.toml");
    if !cargo_toml_path.exists() {
        return None;
    }

    let content = std::fs::read_to_string(&cargo_toml_path).ok()?;
    let cargo: CargoToml = toml::from_str(&content).ok()?;
    let workspace = cargo.workspace?;
    let member_patterns = workspace.members.unwrap_or_default();

    if member_patterns.is_empty() {
        return None;
    }

    let mut members = Vec::new();
    for pattern in &member_patterns {
        let full_pattern = project_dir.join(pattern);
        let pattern_str = full_pattern.to_string_lossy();

        // Use glob to expand patterns like `crates/*`
        if let Ok(paths) = glob::glob(&pattern_str) {
            for path in paths.flatten() {
                if path.is_dir() {
                    if let Some(name) = get_crate_name(&path) {
                        members.push(WorkspaceMember { name, path });
                    }
                }
            }
        }
    }

    if members.is_empty() {
        None
    } else {
        Some(WorkspaceInfo {
            root: project_dir.to_path_buf(),
            members,
        })
    }
}

/// Return names of workspace crates touched by `git diff --name-only HEAD`.
pub fn get_changed_crates(project_dir: &Path) -> Result<Vec<String>> {
    let output = std::process::Command::new("git")
        .args(["diff", "--name-only", "HEAD"])
        .current_dir(project_dir)
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    let workspace = match detect_workspace(project_dir) {
        Some(w) => w,
        None => return Ok(vec![]),
    };

    let mut changed = Vec::new();
    for line in stdout.lines() {
        let file_path = project_dir.join(line);
        for member in &workspace.members {
            if file_path.starts_with(&member.path) && !changed.contains(&member.name) {
                changed.push(member.name.clone());
            }
        }
    }

    Ok(changed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn create_crate(workspace_root: &Path, rel_path: &str, name: &str) {
        let dir = workspace_root.join(rel_path);
        std::fs::create_dir_all(&dir).unwrap();
        let mut f = std::fs::File::create(dir.join("Cargo.toml")).unwrap();
        write!(
            f,
            "[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )
        .unwrap();
    }

    fn create_workspace(root: &Path, members: &[&str]) {
        let members_str: Vec<String> = members.iter().map(|m| format!("\"{m}\"")).collect();
        let content = format!("[workspace]\nmembers = [{}]\n", members_str.join(", "));
        std::fs::write(root.join("Cargo.toml"), content).unwrap();
    }

    #[test]
    fn test_detect_workspace_single_member() {
        let dir = tempfile::tempdir().unwrap();
        create_workspace(dir.path(), &["crate-a"]);
        create_crate(dir.path(), "crate-a", "crate-a");

        let info = detect_workspace(dir.path());
        assert!(info.is_some());
        let info = info.unwrap();
        assert_eq!(info.members.len(), 1);
        assert_eq!(info.members[0].name, "crate-a");
    }

    #[test]
    fn test_detect_workspace_multiple_members() {
        let dir = tempfile::tempdir().unwrap();
        create_workspace(dir.path(), &["crate-a", "crate-b"]);
        create_crate(dir.path(), "crate-a", "crate-a");
        create_crate(dir.path(), "crate-b", "crate-b");

        let info = detect_workspace(dir.path()).unwrap();
        assert_eq!(info.members.len(), 2);
    }

    #[test]
    fn test_detect_workspace_no_workspace_section() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("Cargo.toml"),
            "[package]\nname = \"solo\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();

        assert!(detect_workspace(dir.path()).is_none());
    }

    #[test]
    fn test_detect_workspace_no_cargo_toml() {
        let dir = tempfile::tempdir().unwrap();
        assert!(detect_workspace(dir.path()).is_none());
    }

    #[test]
    fn test_detect_workspace_glob_expansion() {
        let dir = tempfile::tempdir().unwrap();
        // Use a glob pattern `crates/*`
        create_workspace(dir.path(), &["crates/*"]);
        create_crate(dir.path(), "crates/alpha", "alpha");
        create_crate(dir.path(), "crates/beta", "beta");

        let info = detect_workspace(dir.path()).unwrap();
        assert_eq!(info.members.len(), 2);
        let mut names: Vec<_> = info.members.iter().map(|m| m.name.as_str()).collect();
        names.sort_unstable();
        assert_eq!(names, vec!["alpha", "beta"]);
    }
}
