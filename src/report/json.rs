use crate::tools::ToolReport;
use anyhow::Result;
use serde_json::json;
use std::path::Path;

pub fn write_report(tool_name: &str, report: &ToolReport, path: &Path) -> Result<()> {
    let sections: Vec<_> = report
        .sections
        .iter()
        .map(|s| json!({ "title": s.title, "content": s.content }))
        .collect();

    let obj = json!({
        "tool": tool_name,
        "status": format!("{:?}", report.status),
        "sections": sections
    });

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, serde_json::to_string_pretty(&obj)?)?;
    Ok(())
}
