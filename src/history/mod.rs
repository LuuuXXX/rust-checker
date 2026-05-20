use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::report::{ToolReport, ToolStatus};

/// A single persisted run entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub timestamp: String,
    pub tools: Vec<HistoryToolResult>,
}

/// Per-tool result stored in history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryToolResult {
    pub tool_name: String,
    pub status: String,
    pub summary: String,
}

fn status_to_str(s: &ToolStatus) -> &'static str {
    match s {
        ToolStatus::Ok => "ok",
        ToolStatus::Warn => "warn",
        ToolStatus::Error => "error",
        ToolStatus::Skipped => "skipped",
    }
}

/// Save the current run results as a new history entry and prune old entries.
pub fn save_history(reports: &[ToolReport], history_base: &Path, max_entries: u32) -> Result<()> {
    let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
    let entry_dir = history_base.join(&timestamp);
    std::fs::create_dir_all(&entry_dir)?;

    let entry = HistoryEntry {
        timestamp: timestamp.clone(),
        tools: reports
            .iter()
            .map(|r| HistoryToolResult {
                tool_name: r.tool_name.clone(),
                status: status_to_str(&r.status).to_string(),
                summary: r.summary.clone(),
            })
            .collect(),
    };

    let json = serde_json::to_string_pretty(&entry)?;
    std::fs::write(entry_dir.join("result.json"), json)?;

    prune_history(history_base, max_entries)?;

    Ok(())
}

/// List history entry directories, sorted oldest-first.
pub fn list_history_dirs(history_base: &Path) -> Result<Vec<PathBuf>> {
    if !history_base.exists() {
        return Ok(vec![]);
    }
    let mut entries: Vec<PathBuf> = std::fs::read_dir(history_base)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_dir() && p.join("result.json").exists())
        .collect();
    entries.sort();
    Ok(entries)
}

/// Load a history entry from a directory.
pub fn load_history_entry(dir: &Path) -> Result<HistoryEntry> {
    let content = std::fs::read_to_string(dir.join("result.json"))?;
    let entry: HistoryEntry = serde_json::from_str(&content)?;
    Ok(entry)
}

fn prune_history(history_base: &Path, max_entries: u32) -> Result<()> {
    let entries = list_history_dirs(history_base)?;
    let max = max_entries as usize;
    if entries.len() > max {
        let to_remove = &entries[..entries.len() - max];
        for path in to_remove {
            std::fs::remove_dir_all(path)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::ToolStatus;

    fn make_report(name: &str, status: ToolStatus) -> ToolReport {
        ToolReport {
            tool_name: name.to_string(),
            status,
            summary: format!("{name} summary"),
            output_path: format!("quality/{name}.md"),
            markdown_content: String::new(),
        }
    }

    #[test]
    fn test_save_and_load_history() {
        let dir = tempfile::tempdir().unwrap();
        let history_dir = dir.path().join("history");
        let reports = vec![
            make_report("build", ToolStatus::Ok),
            make_report("test", ToolStatus::Error),
        ];
        save_history(&reports, &history_dir, 10).unwrap();

        let entries = list_history_dirs(&history_dir).unwrap();
        assert_eq!(entries.len(), 1);

        let entry = load_history_entry(&entries[0]).unwrap();
        assert_eq!(entry.tools.len(), 2);
        assert_eq!(entry.tools[0].tool_name, "build");
        assert_eq!(entry.tools[0].status, "ok");
        assert_eq!(entry.tools[1].tool_name, "test");
        assert_eq!(entry.tools[1].status, "error");
    }

    #[test]
    fn test_history_prune_keeps_max_entries() {
        let dir = tempfile::tempdir().unwrap();
        let history_dir = dir.path().join("history");
        std::fs::create_dir_all(&history_dir).unwrap();

        // Pre-create 5 synthetic entries directly (no real-clock timestamps needed).
        let json = r#"{"timestamp":"T","tools":[]}"#;
        for i in 0..5u32 {
            let entry_dir = history_dir.join(format!("20260101-{:06}", i));
            std::fs::create_dir_all(&entry_dir).unwrap();
            std::fs::write(entry_dir.join("result.json"), json).unwrap();
        }

        // save_history creates one more entry and then prunes to max_entries=3.
        let reports = vec![make_report("build", ToolStatus::Ok)];
        save_history(&reports, &history_dir, 3).unwrap();

        let entries = list_history_dirs(&history_dir).unwrap();
        assert!(
            entries.len() <= 3,
            "expected <= 3 entries after prune, got {}",
            entries.len()
        );
    }

    #[test]
    fn test_list_history_empty() {
        let dir = tempfile::tempdir().unwrap();
        let history_dir = dir.path().join("history");
        let entries = list_history_dirs(&history_dir).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn test_status_str_roundtrip() {
        let reports = vec![
            make_report("a", ToolStatus::Ok),
            make_report("b", ToolStatus::Warn),
            make_report("c", ToolStatus::Error),
            make_report("d", ToolStatus::Skipped),
        ];
        let dir = tempfile::tempdir().unwrap();
        let history_dir = dir.path().join("history");
        save_history(&reports, &history_dir, 10).unwrap();

        let dirs = list_history_dirs(&history_dir).unwrap();
        let entry = load_history_entry(&dirs[0]).unwrap();

        let statuses: Vec<_> = entry.tools.iter().map(|t| t.status.as_str()).collect();
        assert_eq!(statuses, vec!["ok", "warn", "error", "skipped"]);
    }
}
