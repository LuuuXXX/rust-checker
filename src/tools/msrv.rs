use crate::tools::{ReportSection, ReportStatus, ToolParser, ToolReport};

pub struct MsrvParser;

impl ToolParser for MsrvParser {
    fn parse(&self, stdout: &str, stderr: &str, exit_code: i32) -> ToolReport {
        let combined = format!("{}\n{}", stdout, stderr);
        let msrv = combined
            .lines()
            .find(|l| l.contains("MSRV") || l.contains("Minimum Supported"))
            .map(|l| l.trim().to_string())
            .unwrap_or_else(|| "Not determined".to_string());

        let status = if exit_code != 0 {
            ReportStatus::Error
        } else {
            ReportStatus::Ok
        };

        let content = format!(
            "| Field | Value |\n|-------|-------|\n| MSRV | {} |",
            msrv
        );

        ToolReport {
            status,
            sections: vec![ReportSection {
                title: "Minimum Supported Rust Version".to_string(),
                content,
            }],
        }
    }
}
