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

    let ok = reports
        .iter()
        .filter(|r| r.status == ToolStatus::Ok)
        .count();
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::{ToolReport, ToolStatus};

    fn make_report(name: &str, status: ToolStatus) -> ToolReport {
        ToolReport {
            tool_name: name.to_string(),
            status,
            summary: "test summary".to_string(),
            output_path: format!("quality/{name}.md"),
            markdown_content: String::new(),
        }
    }

    #[test]
    fn test_build_ci_json_structure() {
        let reports = vec![
            make_report("build", ToolStatus::Ok),
            make_report("test", ToolStatus::Error),
            make_report("clippy", ToolStatus::Warn),
            make_report("fmt", ToolStatus::Skipped),
        ];
        let json = build_ci_json(&reports, "2026-01-01T00:00:00");

        assert_eq!(json["timestamp"], "2026-01-01T00:00:00");
        assert_eq!(json["summary"]["total"], 4);
        assert_eq!(json["summary"]["ok"], 1);
        assert_eq!(json["summary"]["error"], 1);
        assert_eq!(json["summary"]["warn"], 1);
        assert_eq!(json["summary"]["skipped"], 1);
        assert_eq!(json["tools"].as_array().unwrap().len(), 4);
    }

    #[test]
    fn test_build_ci_json_tool_status_strings() {
        let reports = vec![
            make_report("a", ToolStatus::Ok),
            make_report("b", ToolStatus::Warn),
            make_report("c", ToolStatus::Error),
            make_report("d", ToolStatus::Skipped),
        ];
        let json = build_ci_json(&reports, "ts");
        let tools = json["tools"].as_array().unwrap();
        assert_eq!(tools[0]["status"], "ok");
        assert_eq!(tools[1]["status"], "warn");
        assert_eq!(tools[2]["status"], "error");
        assert_eq!(tools[3]["status"], "skipped");
    }

    #[test]
    fn test_build_ci_json_empty() {
        let json = build_ci_json(&[], "ts");
        assert_eq!(json["summary"]["total"], 0);
        assert!(json["tools"].as_array().unwrap().is_empty());
    }

    #[test]
    fn test_build_ci_json_tool_fields() {
        let reports = vec![make_report("build", ToolStatus::Ok)];
        let json = build_ci_json(&reports, "ts");
        let tool = &json["tools"][0];
        assert_eq!(tool["tool"], "build");
        assert_eq!(tool["status"], "ok");
        assert_eq!(tool["summary"], "test summary");
        assert_eq!(tool["output_path"], "quality/build.md");
    }
}
