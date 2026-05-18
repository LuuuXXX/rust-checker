use anyhow::Result;
use std::path::Path;

use crate::report::ToolReport;

pub fn write_tool_report(report_dir: &Path, report: &ToolReport) -> Result<()> {
    let path = report_dir.join(&report.output_path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, &report.markdown_content)?;
    Ok(())
}
