use std::collections::BTreeMap;

use crate::history::{HistoryEntry, HistoryToolResult};

/// Diff information for a single tool between two runs.
#[derive(Debug)]
pub struct ToolDiff {
    pub tool_name: String,
    pub old_status: Option<String>,
    pub new_status: Option<String>,
    pub old_summary: Option<String>,
    pub new_summary: Option<String>,
}

impl ToolDiff {
    pub fn status_changed(&self) -> bool {
        self.old_status != self.new_status
    }

    pub fn is_new_tool(&self) -> bool {
        self.old_status.is_none() && self.new_status.is_some()
    }

    pub fn is_removed_tool(&self) -> bool {
        self.old_status.is_some() && self.new_status.is_none()
    }
}

/// Compare two history entries and return per-tool diffs.
pub fn diff_entries(old: &HistoryEntry, new: &HistoryEntry) -> Vec<ToolDiff> {
    let mut old_map: BTreeMap<&str, &HistoryToolResult> = BTreeMap::new();
    let mut new_map: BTreeMap<&str, &HistoryToolResult> = BTreeMap::new();

    for t in &old.tools {
        old_map.insert(&t.tool_name, t);
    }
    for t in &new.tools {
        new_map.insert(&t.tool_name, t);
    }

    let mut all_names: Vec<&str> = old_map
        .keys()
        .chain(new_map.keys())
        .copied()
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    all_names.sort_unstable();

    all_names
        .iter()
        .map(|name| ToolDiff {
            tool_name: name.to_string(),
            old_status: old_map.get(name).map(|t| t.status.clone()),
            new_status: new_map.get(name).map(|t| t.status.clone()),
            old_summary: old_map.get(name).map(|t| t.summary.clone()),
            new_summary: new_map.get(name).map(|t| t.summary.clone()),
        })
        .collect()
}

fn status_emoji(status: &str) -> &'static str {
    match status {
        "ok" => "✅",
        "warn" => "⚠️",
        "error" => "❌",
        "skipped" => "⏭️",
        _ => "❓",
    }
}

fn change_indicator(old: Option<&str>, new: Option<&str>) -> &'static str {
    match (old, new) {
        (None, Some(_)) => "➕ 新增",
        (Some(_), None) => "🗑️ 移除",
        (Some(o), Some(n)) if o == n => "→",
        (Some(_), Some("ok")) => "↑ 改善",
        (Some(_), Some(_)) => "↓ 劣化",
        _ => "→",
    }
}

/// Format diff output as human-readable text.
pub fn format_diff(diffs: &[ToolDiff], old_timestamp: &str, new_timestamp: &str) -> String {
    let mut out = String::new();

    out.push_str("## 变更对比\n\n");
    out.push_str(&format!("- 旧版本: `{old_timestamp}`\n"));
    out.push_str(&format!("- 新版本: `{new_timestamp}`\n\n"));

    let changed: Vec<&ToolDiff> = diffs.iter().filter(|d| d.status_changed()).collect();
    let unchanged: Vec<&ToolDiff> = diffs.iter().filter(|d| !d.status_changed()).collect();

    if changed.is_empty() {
        out.push_str("✅ 无状态变化\n\n");
    } else {
        out.push_str("### 状态变化\n\n");
        out.push_str("| 工具 | 旧状态 | 变化 | 新状态 |\n");
        out.push_str("|------|--------|------|--------|\n");
        for d in &changed {
            let old_s = d.old_status.as_deref().unwrap_or("N/A");
            let new_s = d.new_status.as_deref().unwrap_or("N/A");
            let indicator = change_indicator(d.old_status.as_deref(), d.new_status.as_deref());
            let old_emoji = if d.old_status.is_some() {
                status_emoji(old_s)
            } else {
                ""
            };
            let new_emoji = if d.new_status.is_some() {
                status_emoji(new_s)
            } else {
                ""
            };
            out.push_str(&format!(
                "| {name} | {old_emoji} {old_s} | {indicator} | {new_emoji} {new_s} |\n",
                name = d.tool_name,
                old_emoji = old_emoji,
                old_s = old_s,
                indicator = indicator,
                new_emoji = new_emoji,
                new_s = new_s
            ));
        }
        out.push('\n');
    }

    if !unchanged.is_empty() {
        out.push_str(&format!("### 未变化 ({} 个工具)\n\n", unchanged.len()));
        for d in &unchanged {
            let s = d.new_status.as_deref().unwrap_or("N/A");
            out.push_str(&format!("- {} {} ({})\n", status_emoji(s), d.tool_name, s));
        }
        out.push('\n');
    }

    out
}

/// Format trend for last N history entries.
pub fn format_trend(entries: &[HistoryEntry]) -> String {
    if entries.is_empty() {
        return "没有历史记录。\n".to_string();
    }

    let mut out = String::new();
    out.push_str("## 历史趋势\n\n");

    // Collect all tool names across entries
    let mut all_tools: Vec<String> = entries
        .iter()
        .flat_map(|e| e.tools.iter().map(|t| t.tool_name.clone()))
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    all_tools.sort_unstable();

    // Header row: timestamps
    out.push_str("| 工具 |");
    for e in entries {
        out.push_str(&format!(" {} |", &e.timestamp));
    }
    out.push('\n');
    out.push_str("|------|");
    for _ in entries {
        out.push_str("------|");
    }
    out.push('\n');

    for tool in &all_tools {
        out.push_str(&format!("| {} |", tool));
        for entry in entries {
            let status = entry
                .tools
                .iter()
                .find(|t| &t.tool_name == tool)
                .map(|t| t.status.as_str())
                .unwrap_or("N/A");
            out.push_str(&format!(" {} {} |", status_emoji(status), status));
        }
        out.push('\n');
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::history::{HistoryEntry, HistoryToolResult};

    fn make_entry(ts: &str, tools: &[(&str, &str)]) -> HistoryEntry {
        HistoryEntry {
            timestamp: ts.to_string(),
            tools: tools
                .iter()
                .map(|(name, status)| HistoryToolResult {
                    tool_name: name.to_string(),
                    status: status.to_string(),
                    summary: format!("{name} summary"),
                })
                .collect(),
        }
    }

    #[test]
    fn test_diff_no_changes() {
        let old = make_entry("20260101-100000", &[("build", "ok"), ("test", "ok")]);
        let new = make_entry("20260101-110000", &[("build", "ok"), ("test", "ok")]);
        let diffs = diff_entries(&old, &new);
        assert!(diffs.iter().all(|d| !d.status_changed()));
    }

    #[test]
    fn test_diff_status_changed() {
        let old = make_entry("20260101-100000", &[("build", "ok"), ("test", "ok")]);
        let new = make_entry("20260101-110000", &[("build", "ok"), ("test", "error")]);
        let diffs = diff_entries(&old, &new);
        let test_diff = diffs.iter().find(|d| d.tool_name == "test").unwrap();
        assert!(test_diff.status_changed());
        assert_eq!(test_diff.old_status.as_deref(), Some("ok"));
        assert_eq!(test_diff.new_status.as_deref(), Some("error"));
    }

    #[test]
    fn test_diff_new_tool() {
        let old = make_entry("20260101-100000", &[("build", "ok")]);
        let new = make_entry("20260101-110000", &[("build", "ok"), ("clippy", "warn")]);
        let diffs = diff_entries(&old, &new);
        let clippy = diffs.iter().find(|d| d.tool_name == "clippy").unwrap();
        assert!(clippy.is_new_tool());
        assert!(clippy.status_changed());
    }

    #[test]
    fn test_diff_removed_tool() {
        let old = make_entry("20260101-100000", &[("build", "ok"), ("audit", "error")]);
        let new = make_entry("20260101-110000", &[("build", "ok")]);
        let diffs = diff_entries(&old, &new);
        let audit = diffs.iter().find(|d| d.tool_name == "audit").unwrap();
        assert!(audit.is_removed_tool());
    }

    #[test]
    fn test_format_diff_contains_timestamps() {
        let old = make_entry("20260101-100000", &[("build", "ok")]);
        let new = make_entry("20260101-110000", &[("build", "ok")]);
        let diffs = diff_entries(&old, &new);
        let output = format_diff(&diffs, &old.timestamp, &new.timestamp);
        assert!(output.contains("20260101-100000"));
        assert!(output.contains("20260101-110000"));
    }

    #[test]
    fn test_format_diff_shows_no_change() {
        let old = make_entry("20260101-100000", &[("build", "ok")]);
        let new = make_entry("20260101-110000", &[("build", "ok")]);
        let diffs = diff_entries(&old, &new);
        let output = format_diff(&diffs, &old.timestamp, &new.timestamp);
        assert!(output.contains("无状态变化"));
    }

    #[test]
    fn test_format_diff_shows_changed_status() {
        let old = make_entry("20260101-100000", &[("test", "ok")]);
        let new = make_entry("20260101-110000", &[("test", "error")]);
        let diffs = diff_entries(&old, &new);
        let output = format_diff(&diffs, &old.timestamp, &new.timestamp);
        assert!(output.contains("状态变化"));
        assert!(output.contains("test"));
    }

    #[test]
    fn test_format_trend_empty() {
        let output = format_trend(&[]);
        assert!(output.contains("没有历史记录"));
    }

    #[test]
    fn test_format_trend_multiple_entries() {
        let entries = vec![
            make_entry("20260101-100000", &[("build", "ok"), ("test", "ok")]),
            make_entry("20260101-110000", &[("build", "ok"), ("test", "error")]),
        ];
        let output = format_trend(&entries);
        assert!(output.contains("历史趋势"));
        assert!(output.contains("build"));
        assert!(output.contains("test"));
        assert!(output.contains("20260101-100000"));
        assert!(output.contains("20260101-110000"));
    }
}
