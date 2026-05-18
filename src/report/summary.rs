use crate::engine::runner::{RunSummary, ToolStatus};
use anyhow::Result;
use std::path::PathBuf;

pub fn write_summary(summary: &RunSummary) -> Result<()> {
    let dir = PathBuf::from(".localcheck/reports");
    std::fs::create_dir_all(&dir)?;

    let mut md = format!(
        "# Run Summary\n\nTimestamp: {}\n\n",
        summary.timestamp
    );
    md.push_str("| Tool | Status | Duration |\n|------|--------|----------|\n");
    for r in &summary.tool_results {
        let symbol = match r.status {
            ToolStatus::Ok => "✅",
            ToolStatus::Warn => "⚠️",
            ToolStatus::Error => "❌",
            ToolStatus::Skipped => "⏭️",
        };
        md.push_str(&format!(
            "| {} | {} | {}ms |\n",
            r.name, symbol, r.duration_ms
        ));
    }
    std::fs::write(dir.join("summary.md"), &md)?;

    // HTML version
    let mut rows = String::new();
    for r in &summary.tool_results {
        let symbol = match r.status {
            ToolStatus::Ok => "✅",
            ToolStatus::Warn => "⚠️",
            ToolStatus::Error => "❌",
            ToolStatus::Skipped => "⏭️",
        };
        rows.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}ms</td></tr>\n",
            r.name, symbol, r.duration_ms
        ));
    }
    let html = format!(
        "<!DOCTYPE html>\n<html>\n<head><title>Run Summary</title>\
        <style>body{{font-family:sans-serif;max-width:900px;margin:auto;padding:20px}}\
        table{{border-collapse:collapse;width:100%}}td,th{{border:1px solid #ddd;padding:8px}}\
        th{{background:#f2f2f2}}</style>\n</head>\n\
        <body>\n<h1>Run Summary</h1>\n<p>Timestamp: {}</p>\n\
        <table><tr><th>Tool</th><th>Status</th><th>Duration</th></tr>\n{}</table>\n\
        </body>\n</html>",
        summary.timestamp, rows
    );
    std::fs::write(dir.join("summary.html"), html)?;

    Ok(())
}
