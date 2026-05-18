use anyhow::Result;
use std::path::Path;

use crate::report::ToolReport;

pub fn write_tool_report_html(report_dir: &Path, report: &ToolReport) -> Result<()> {
    let html_path = report_dir.join(report.output_path.replace(".md", ".html"));
    if let Some(parent) = html_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let body = markdown_to_html(&report.markdown_content);
    let html = format!(
        "<!DOCTYPE html><html><head><meta charset='utf-8'><title>{}</title></head><body>{}</body></html>",
        report.tool_name, body
    );
    std::fs::write(html_path, html)?;
    Ok(())
}

fn markdown_to_html(md: &str) -> String {
    format!(
        "<pre>{}</pre>",
        md.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
    )
}
