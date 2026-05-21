use anyhow::Result;
use std::path::Path;

use crate::report::{ToolReport, ToolStatus};

pub fn write_tool_report_html(report_dir: &Path, report: &ToolReport) -> Result<()> {
    let html_path = report_dir.join(report.output_path.replace(".md", ".html"));
    if let Some(parent) = html_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let body = markdown_to_html(&report.markdown_content);
    let status_class = match report.status {
        ToolStatus::Ok => "ok",
        ToolStatus::Warn => "warn",
        ToolStatus::Error => "error",
        ToolStatus::Skipped => "skipped",
    };
    let html = format!(
        r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width,initial-scale=1">
  <title>{name} — rust-checker</title>
  <style>
    body{{font-family:system-ui,sans-serif;max-width:860px;margin:2em auto;padding:0 1em;color:#222}}
    h1{{font-size:1.6em;margin-bottom:.3em}}
    .badge{{display:inline-block;padding:.2em .6em;border-radius:4px;font-size:.85em;font-weight:600;color:#fff}}
    .ok{{background:#22863a}}.warn{{background:#b08800}}.error{{background:#cb2431}}.skipped{{background:#6a737d}}
    pre{{background:#f6f8fa;padding:1em;border-radius:6px;overflow-x:auto;font-size:.9em;white-space:pre-wrap}}
    a{{color:#0366d6}}
  </style>
</head>
<body>
  <h1>{name} <span class="badge {status_class}">{status_label}</span></h1>
  <p><strong>摘要：</strong>{summary}</p>
  <hr>
  {body}
  <p><a href="../summary.html">← 返回汇总报告</a></p>
</body>
</html>"#,
        name = html_escape(&report.tool_name),
        status_class = status_class,
        status_label = html_escape(report.status.label()),
        summary = html_escape(&report.summary),
        body = body,
    );
    std::fs::write(html_path, html)?;
    Ok(())
}

pub fn write_summary_html(report_dir: &Path, reports: &[ToolReport]) -> Result<()> {
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");

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

    let mut cards = String::new();
    for r in reports {
        let status_class = match r.status {
            ToolStatus::Ok => "ok",
            ToolStatus::Warn => "warn",
            ToolStatus::Error => "error",
            ToolStatus::Skipped => "skipped",
        };
        cards.push_str(&format!(
            r#"<div class="card {status_class}">
  <div class="card-header">
    <span class="tool-name">{name}</span>
    <span class="badge {status_class}">{emoji} {label}</span>
  </div>
  <div class="card-summary">{summary}</div>
  <a class="card-link" href="{link}">查看报告 →</a>
</div>
"#,
            status_class = status_class,
            name = html_escape(&r.tool_name),
            emoji = r.status.emoji(),
            label = html_escape(r.status.label()),
            summary = html_escape(&r.summary),
            link = html_escape(&r.output_path.replace(".md", ".html")),
        ));
    }

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width,initial-scale=1">
  <title>rust-checker — 质量检查汇总报告</title>
  <style>
    *{{box-sizing:border-box}}
    body{{font-family:system-ui,sans-serif;background:#f0f2f5;margin:0;padding:2em 1em;color:#222}}
    h1{{text-align:center;font-size:1.8em;margin-bottom:.2em}}
    .subtitle{{text-align:center;color:#555;margin-bottom:2em;font-size:.95em}}
    .stat-bar{{display:flex;justify-content:center;gap:1.5em;margin-bottom:2em;flex-wrap:wrap}}
    .stat{{background:#fff;border-radius:8px;padding:.8em 1.5em;text-align:center;box-shadow:0 1px 4px rgba(0,0,0,.1)}}
    .stat .num{{font-size:2em;font-weight:700;line-height:1}}
    .stat .lbl{{font-size:.8em;color:#555;margin-top:.2em}}
    .stat.ok .num{{color:#22863a}}.stat.warn .num{{color:#b08800}}
    .stat.error .num{{color:#cb2431}}.stat.skipped .num{{color:#6a737d}}
    .grid{{display:grid;grid-template-columns:repeat(auto-fill,minmax(260px,1fr));gap:1em;max-width:1100px;margin:auto}}
    .card{{background:#fff;border-radius:8px;padding:1em 1.2em;box-shadow:0 1px 4px rgba(0,0,0,.1);border-left:4px solid #ccc;display:flex;flex-direction:column;gap:.4em}}
    .card.ok{{border-color:#22863a}}.card.warn{{border-color:#b08800}}
    .card.error{{border-color:#cb2431}}.card.skipped{{border-color:#6a737d}}
    .card-header{{display:flex;justify-content:space-between;align-items:center}}
    .tool-name{{font-weight:700;font-size:1.05em}}
    .badge{{display:inline-block;padding:.15em .55em;border-radius:4px;font-size:.75em;font-weight:600;color:#fff}}
    .ok .badge{{background:#22863a}}.warn .badge{{background:#b08800}}
    .error .badge{{background:#cb2431}}.skipped .badge{{background:#6a737d}}
    .card-summary{{font-size:.9em;color:#444;flex:1}}
    .card-link{{font-size:.85em;color:#0366d6;text-decoration:none;align-self:flex-end}}
    .card-link:hover{{text-decoration:underline}}
  </style>
</head>
<body>
  <h1>🦀 rust-checker 质量检查报告</h1>
  <p class="subtitle">生成时间：{timestamp}</p>

  <div class="stat-bar">
    <div class="stat ok"><div class="num">{ok}</div><div class="lbl">✅ 通过</div></div>
    <div class="stat warn"><div class="num">{warn}</div><div class="lbl">⚠️ 警告</div></div>
    <div class="stat error"><div class="num">{err}</div><div class="lbl">❌ 失败</div></div>
    <div class="stat skipped"><div class="num">{skip}</div><div class="lbl">⏭️ 跳过</div></div>
  </div>

  <div class="grid">
{cards}  </div>
</body>
</html>
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
    fn test_write_tool_report_html_contains_tool_name() {
        let dir = tempfile::tempdir().unwrap();
        let report = make_report("clippy", ToolStatus::Warn);
        write_tool_report_html(dir.path(), &report).unwrap();
        let content =
            std::fs::read_to_string(dir.path().join("quality").join("clippy.html")).unwrap();
        assert!(content.contains("clippy"));
        assert!(content.contains("warn"));
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
        assert!(content.contains(">1<")); // ok count
        assert!(content.contains("失败"));
        assert!(content.contains("警告"));
        assert!(content.contains("跳过"));
    }

    #[test]
    fn test_write_summary_html_has_stat_bar() {
        let dir = tempfile::tempdir().unwrap();
        let reports = vec![
            make_report("build", ToolStatus::Ok),
            make_report("test", ToolStatus::Ok),
        ];
        write_summary_html(dir.path(), &reports).unwrap();
        let content = std::fs::read_to_string(dir.path().join("summary.html")).unwrap();
        assert!(content.contains("stat-bar"));
        assert!(content.contains("grid"));
    }

    #[test]
    fn test_html_escape() {
        assert_eq!(html_escape("<b>"), "&lt;b&gt;");
        assert_eq!(html_escape("a & b"), "a &amp; b");
        assert_eq!(html_escape("\"hi\""), "&quot;hi&quot;");
    }
}
