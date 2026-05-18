pub mod html;
pub mod json;
pub mod markdown;

use anyhow::Result;
use std::fmt::Write as FmtWrite;
use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub enum ReportFormat {
    Markdown,
    Html,
    Json,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ToolStatus {
    Ok,
    Warn,
    Error,
    Skipped,
}

impl ToolStatus {
    pub fn emoji(&self) -> &'static str {
        match self {
            ToolStatus::Ok => "✅",
            ToolStatus::Warn => "⚠️",
            ToolStatus::Error => "❌",
            ToolStatus::Skipped => "⏭️",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            ToolStatus::Ok => "通过",
            ToolStatus::Warn => "警告",
            ToolStatus::Error => "失败",
            ToolStatus::Skipped => "跳过",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ToolReport {
    pub tool_name: String,
    pub status: ToolStatus,
    pub summary: String,
    pub output_path: String,
    pub markdown_content: String,
}

pub fn write_summary(report_dir: &Path, reports: &[ToolReport]) -> Result<()> {
    let mut content = String::new();
    writeln!(content, "# 质量检查汇总报告\n")?;
    writeln!(
        content,
        "生成时间: {}\n",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
    )?;
    writeln!(content, "## 工具检查结果\n")?;
    writeln!(content, "| 工具 | 状态 | 摘要 | 报告 |")?;
    writeln!(content, "|-----|------|------|------|")?;
    for r in reports {
        writeln!(
            content,
            "| {} | {} {} | {} | [查看]({}) |",
            r.tool_name,
            r.status.emoji(),
            r.status.label(),
            r.summary,
            r.output_path,
        )?;
    }
    writeln!(content)?;

    let ok = reports
        .iter()
        .filter(|r| r.status == ToolStatus::Ok)
        .count();
    let warn = reports
        .iter()
        .filter(|r| r.status == ToolStatus::Warn)
        .count();
    let err = reports
        .iter()
        .filter(|r| r.status == ToolStatus::Error)
        .count();
    let skip = reports
        .iter()
        .filter(|r| r.status == ToolStatus::Skipped)
        .count();

    writeln!(content, "## 统计\n")?;
    writeln!(content, "- ✅ 通过: {ok}")?;
    writeln!(content, "- ⚠️ 警告: {warn}")?;
    writeln!(content, "- ❌ 失败: {err}")?;
    writeln!(content, "- ⏭️ 跳过: {skip}")?;

    std::fs::write(report_dir.join("summary.md"), content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_report(name: &str, status: ToolStatus, summary: &str) -> ToolReport {
        ToolReport {
            tool_name: name.to_string(),
            status,
            summary: summary.to_string(),
            output_path: format!("quality/{name}.md"),
            markdown_content: format!("# {name}\n\n{summary}\n"),
        }
    }

    #[test]
    fn test_tool_status_emoji() {
        assert_eq!(ToolStatus::Ok.emoji(), "✅");
        assert_eq!(ToolStatus::Warn.emoji(), "⚠️");
        assert_eq!(ToolStatus::Error.emoji(), "❌");
        assert_eq!(ToolStatus::Skipped.emoji(), "⏭️");
    }

    #[test]
    fn test_tool_status_label() {
        assert_eq!(ToolStatus::Ok.label(), "通过");
        assert_eq!(ToolStatus::Warn.label(), "警告");
        assert_eq!(ToolStatus::Error.label(), "失败");
        assert_eq!(ToolStatus::Skipped.label(), "跳过");
    }

    #[test]
    fn test_write_summary_creates_file() {
        let dir = tempfile::tempdir().unwrap();
        let reports = vec![
            make_report("build", ToolStatus::Ok, "构建成功"),
            make_report("test", ToolStatus::Error, "2个失败"),
            make_report("clippy", ToolStatus::Warn, "3个警告"),
            make_report("fmt", ToolStatus::Skipped, "已跳过"),
        ];
        write_summary(dir.path(), &reports).unwrap();

        let summary_path = dir.path().join("summary.md");
        assert!(summary_path.exists());

        let content = std::fs::read_to_string(summary_path).unwrap();
        assert!(content.contains("build"));
        assert!(content.contains("✅"));
        assert!(content.contains("❌"));
        assert!(content.contains("⚠️"));
        assert!(content.contains("⏭️"));
    }

    #[test]
    fn test_write_summary_counts() {
        let dir = tempfile::tempdir().unwrap();
        let reports = vec![
            make_report("a", ToolStatus::Ok, "ok"),
            make_report("b", ToolStatus::Ok, "ok"),
            make_report("c", ToolStatus::Error, "err"),
            make_report("d", ToolStatus::Warn, "warn"),
            make_report("e", ToolStatus::Skipped, "skip"),
        ];
        write_summary(dir.path(), &reports).unwrap();
        let content = std::fs::read_to_string(dir.path().join("summary.md")).unwrap();
        assert!(content.contains("通过: 2"));
        assert!(content.contains("警告: 1"));
        assert!(content.contains("失败: 1"));
        assert!(content.contains("跳过: 1"));
    }

    #[test]
    fn test_write_summary_empty_reports() {
        let dir = tempfile::tempdir().unwrap();
        write_summary(dir.path(), &[]).unwrap();
        assert!(dir.path().join("summary.md").exists());
    }
}
