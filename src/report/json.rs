use crate::report::{ToolReport, ToolStatus};
use serde_json::{json, Value};

pub fn build_ci_json(reports: &[ToolReport], timestamp: &str) -> Value {
    let tools: Vec<Value> = reports
        .iter()
        .map(|r| {
            json!({
                "tool": r.tool_name,
                "status": match r.status {
                    ToolStatus::Ok => "ok",
                    ToolStatus::Warn => "warn",
                    ToolStatus::Error => "error",
                    ToolStatus::Skipped => "skipped",
                },
                "summary": r.summary,
                "output_path": r.output_path,
            })
        })
        .collect();

    let ok = reports.iter().filter(|r| r.status == ToolStatus::Ok).count();
    let err = reports
        .iter()
        .filter(|r| r.status == ToolStatus::Error)
        .count();
    let warn = reports
        .iter()
        .filter(|r| r.status == ToolStatus::Warn)
        .count();
    let skip = reports
        .iter()
        .filter(|r| r.status == ToolStatus::Skipped)
        .count();

    json!({
        "timestamp": timestamp,
        "summary": {
            "total": reports.len(),
            "ok": ok,
            "warn": warn,
            "error": err,
            "skipped": skip,
        },
        "tools": tools,
    })
}
