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

    let ok = reports.iter().filter(|r| r.status == ToolStatus::Ok).count();
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
