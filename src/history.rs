use crate::engine::runner::{RunSummary, ToolRunResult, ToolStatus};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub timestamp: String,
    pub tool_results: Vec<ToolRunResult>,
}

pub fn history_dir() -> PathBuf {
    PathBuf::from(".localcheck/history")
}

pub fn save_run(summary: &RunSummary) -> Result<()> {
    let ts = summary.timestamp.format("%Y%m%d-%H%M%S").to_string();
    let dir = history_dir().join(&ts);
    std::fs::create_dir_all(&dir)?;
    let json = serde_json::to_string_pretty(summary)?;
    std::fs::write(dir.join("summary.json"), json)?;
    Ok(())
}

pub fn load_history() -> Result<Vec<HistoryEntry>> {
    let dir = history_dir();
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut entries = Vec::new();
    let mut paths: Vec<_> = std::fs::read_dir(&dir)?
        .filter_map(|e| e.ok())
        .collect();
    paths.sort_by_key(|e| e.file_name());

    for entry in paths {
        let summary_file = entry.path().join("summary.json");
        if summary_file.exists() {
            let content = std::fs::read_to_string(&summary_file)?;
            if let Ok(summary) = serde_json::from_str::<RunSummary>(&content) {
                entries.push(HistoryEntry {
                    timestamp: summary.timestamp.format("%Y%m%d-%H%M%S").to_string(),
                    tool_results: summary.tool_results,
                });
            }
        }
    }
    Ok(entries)
}

pub fn prune_history(max_entries: usize) -> Result<()> {
    let dir = history_dir();
    if !dir.exists() {
        return Ok(());
    }
    let mut paths: Vec<_> = std::fs::read_dir(&dir)?
        .filter_map(|e| e.ok())
        .collect();
    paths.sort_by_key(|e| e.file_name());

    if paths.len() > max_entries {
        let to_remove = paths.len() - max_entries;
        for entry in paths.iter().take(to_remove) {
            std::fs::remove_dir_all(entry.path())?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prune_logic() {
        // Test that prune_history handles empty directories gracefully
        let result = prune_history(10);
        // Should not error even if dir doesn't exist
        assert!(result.is_ok());
    }

    #[test]
    fn test_history_entry_serialization() {
        let entry = HistoryEntry {
            timestamp: "20240101-120000".to_string(),
            tool_results: vec![ToolRunResult {
                name: "build".to_string(),
                status: ToolStatus::Ok,
                duration_ms: 1000,
                output: "ok".to_string(),
                skipped_reason: None,
            }],
        };
        let json = serde_json::to_string(&entry).unwrap();
        let parsed: HistoryEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.timestamp, "20240101-120000");
        assert_eq!(parsed.tool_results.len(), 1);
    }
}
