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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::{ToolReport, ToolStatus};

    fn make_report(name: &str, path: &str, content: &str) -> ToolReport {
        ToolReport {
            tool_name: name.to_string(),
            status: ToolStatus::Ok,
            summary: "ok".to_string(),
            output_path: path.to_string(),
            markdown_content: content.to_string(),
        }
    }

    #[test]
    fn test_write_tool_report_creates_file() {
        let dir = tempfile::tempdir().unwrap();
        let report = make_report("build", "quality/build.md", "# Build\n\nOK\n");
        write_tool_report(dir.path(), &report).unwrap();

        let expected = dir.path().join("quality").join("build.md");
        assert!(expected.exists());
        let content = std::fs::read_to_string(expected).unwrap();
        assert_eq!(content, "# Build\n\nOK\n");
    }

    #[test]
    fn test_write_tool_report_creates_subdirectories() {
        let dir = tempfile::tempdir().unwrap();
        let report = make_report("audit", "security/audit.md", "# Audit\n");
        write_tool_report(dir.path(), &report).unwrap();
        assert!(dir.path().join("security").join("audit.md").exists());
    }

    #[test]
    fn test_write_tool_report_nested_path() {
        let dir = tempfile::tempdir().unwrap();
        let report = make_report("msrv", "compat/msrv.md", "# MSRV\n");
        write_tool_report(dir.path(), &report).unwrap();
        assert!(dir.path().join("compat").join("msrv.md").exists());
    }
}
