use crate::tools::ToolReport;
use anyhow::Result;
use std::path::Path;

pub fn write_report(tool_name: &str, report: &ToolReport, path: &Path) -> Result<()> {
    let mut body = String::new();
    for section in &report.sections {
        body.push_str(&format!(
            "<h2>{}</h2>\n<pre>{}</pre>\n",
            section.title, section.content
        ));
    }
    let html = format!(
        "<!DOCTYPE html>\n<html>\n<head><title>{}</title>\
        <style>body{{font-family:sans-serif;max-width:900px;margin:auto;padding:20px}}\
        h1{{color:#333}}pre{{background:#f5f5f5;padding:10px;border-radius:4px}}</style>\n</head>\n\
        <body>\n<h1>{}</h1>\n{}</body>\n</html>",
        tool_name, tool_name, body
    );
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, html)?;
    Ok(())
}
