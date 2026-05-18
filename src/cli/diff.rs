use anyhow::Result;
use std::path::Path;

use crate::history::{list_history_dirs, load_history_entry};

/// Run `rust-checker diff` in one of three modes.
pub enum DiffMode {
    /// Compare the two most recent runs.
    Latest,
    /// Show last N runs as a trend table.
    Last(usize),
    /// Compare runs within a date range (YYYYMMDD prefix).
    Range { from: String, to: String },
}

pub fn run_diff(project_dir: &Path, mode: DiffMode) -> Result<()> {
    let history_base = project_dir.join(".localcheck").join("history");

    let dirs = list_history_dirs(&history_base)?;
    if dirs.is_empty() {
        anyhow::bail!("没有历史记录。请先运行 `rust-checker run` 以生成历史快照。");
    }

    match mode {
        DiffMode::Latest => {
            if dirs.len() < 2 {
                anyhow::bail!(
                    "历史记录不足两条，无法对比。目前只有 {} 条记录。",
                    dirs.len()
                );
            }
            let old = load_history_entry(&dirs[dirs.len() - 2])?;
            let new = load_history_entry(&dirs[dirs.len() - 1])?;
            let diffs = crate::diff::diff_entries(&old, &new);
            print!(
                "{}",
                crate::diff::format_diff(&diffs, &old.timestamp, &new.timestamp)
            );
        }

        DiffMode::Last(n) => {
            if n < 1 {
                anyhow::bail!("--last 必须 >= 1");
            }
            let start = dirs.len().saturating_sub(n);
            let selected: Vec<_> = dirs[start..].iter().collect();
            let entries: Result<Vec<_>> = selected.iter().map(|d| load_history_entry(d)).collect();
            let entries = entries?;
            print!("{}", crate::diff::format_trend(&entries));
        }

        DiffMode::Range { from, to } => {
            let matching: Vec<_> = dirs
                .iter()
                .filter(|d| {
                    let ts = d
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    ts >= from && ts <= to
                })
                .collect();

            if matching.is_empty() {
                anyhow::bail!("在指定时间范围 [{from}, {to}] 内没有历史记录。");
            }

            if matching.len() < 2 {
                println!("时间范围内只有 1 条记录，展示趋势：");
                let entry = load_history_entry(matching[0])?;
                let entries = vec![entry];
                print!("{}", crate::diff::format_trend(&entries));
            } else {
                // Show as trend across the range
                let entries: Result<Vec<_>> =
                    matching.iter().map(|d| load_history_entry(d)).collect();
                let entries = entries?;
                print!("{}", crate::diff::format_trend(&entries));
            }
        }
    }

    Ok(())
}
