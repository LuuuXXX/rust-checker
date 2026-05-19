use anyhow::Result;
use std::path::Path;

use crate::report::{ToolReport, ToolStatus};

pub fn write_tool_report_html(report_dir: &Path, report: &ToolReport) -> Result<()> {
    let html_path = report_dir.join(report.output_path.replace(".md", ".html"));
    if let Some(parent) = html_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let body = markdown_to_html(&report.markdown_content);
    let html = format!(
        "<!DOCTYPE html><html><head><meta charset='utf-8'><title>{}</title></head><body>{}</body></html>",
        html_escape(&report.tool_name),
        body
    );
    std::fs::write(html_path, html)?;
    Ok(())
}

pub fn write_summary_html(report_dir: &Path, reports: &[ToolReport]) -> Result<()> {
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");

    let mut rows = String::new();
    for r in reports {
        let status_cell = format!("{} {}", r.status.emoji(), r.status.label());
        rows.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td><a href='{}'>查看</a></td></tr>\n",
            html_escape(&r.tool_name),
            html_escape(&status_cell),
            html_escape(&r.summary),
            html_escape(&r.output_path.replace(".md", ".html")),
        ));
    }

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

    let html = format!(
        r#"<!DOCTYPE html>
<html><head><meta charset='utf-8'><title>质量检查汇总报告</title>
<style>body{{font-family:sans-serif;max-width:900px;margin:auto;padding:1em}}
table{{border-collapse:collapse;width:100%}}th,td{{border:1px solid #ccc;padding:8px;text-align:left}}
th{{background:#f4f4f4}}</style></head>
<body>
<h1>质量检查汇总报告</h1>
<p>生成时间: {timestamp}</p>
<table><thead><tr><th>工具</th><th>状态</th><th>摘要</th><th>报告</th></tr></thead>
<tbody>{rows}</tbody></table>
<h2>统计</h2>
<ul>
<li>✅ 通过: {ok}</li>
<li>⚠️ 警告: {warn}</li>
<li>❌ 失败: {err}</li>
<li>⏭️ 跳过: {skip}</li>
</ul>
</body></html>
"#
    );

    std::fs::write(report_dir.join("summary.html"), html)?;
    Ok(())
}

fn markdown_to_html(md: &str) -> String {
    format!("<pre>{}</pre>", html_escape(md))
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::{ToolReport, ToolStatus};

    fn make_report(name: &str, status: ToolStatus) -> ToolReport {
        ToolReport {
            tool_name: name.to_string(),
            status,
            summary: format!("{name} summary"),
            output_path: format!("quality/{name}.md"),
            markdown_content: format!("# {name}\n"),
        }
    }

    #[test]
    fn test_write_tool_report_html_creates_file() {
        let dir = tempfile::tempdir().unwrap();
        let report = make_report("build", ToolStatus::Ok);
        write_tool_report_html(dir.path(), &report).unwrap();
        assert!(dir.path().join("quality").join("build.html").exists());
    }

    #[test]
    fn test_write_summary_html_creates_file() {
        let dir = tempfile::tempdir().unwrap();
        let reports = vec![
            make_report("build", ToolStatus::Ok),
            make_report("test", ToolStatus::Error),
            make_report("clippy", ToolStatus::Warn),
            make_report("fmt", ToolStatus::Skipped),
        ];
        write_summary_html(dir.path(), &reports).unwrap();
        let summary_html = dir.path().join("summary.html");
        assert!(summary_html.exists());
        let content = std::fs::read_to_string(&summary_html).unwrap();
        assert!(content.contains("build"));
        assert!(content.contains("通过: 1"));
        assert!(content.contains("失败: 1"));
        assert!(content.contains("警告: 1"));
        assert!(content.contains("跳过: 1"));
    }

    #[test]
    fn test_html_escape() {
        assert_eq!(html_escape("<b>"), "&lt;b&gt;");
        assert_eq!(html_escape("a & b"), "a &amp; b");
        assert_eq!(html_escape("\"hi\""), "&quot;hi&quot;");
    }
}
