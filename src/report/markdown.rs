use crate::tools::ToolReport;
use anyhow::Result;
use std::path::Path;

pub fn write_report(tool_name: &str, report: &ToolReport, path: &Path) -> Result<()> {
    let mut content = format!("# {}\n\n", tool_name);
    for section in &report.sections {
        content.push_str(&format!("## {}\n\n{}\n\n", section.title, section.content));
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, content)?;
    Ok(())
}
