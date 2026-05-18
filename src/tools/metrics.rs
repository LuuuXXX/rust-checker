use crate::tools::{ReportSection, ReportStatus, ToolParser, ToolReport};

pub struct MetricsParser;

impl ToolParser for MetricsParser {
    fn parse(&self, stdout: &str, _stderr: &str, exit_code: i32) -> ToolReport {
        let line_count = stdout.lines().count();
        let status = if exit_code != 0 {
            ReportStatus::Error
        } else {
            ReportStatus::Ok
        };

        let content = format!(
            "| Metric | Value |\n|--------|-------|\n| Lines of Output | {} |",
            line_count
        );

        ToolReport {
            status,
            sections: vec![ReportSection {
                title: "Metrics".to_string(),
                content,
            }],
        }
    }
}
