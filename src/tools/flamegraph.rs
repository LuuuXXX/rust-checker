use crate::tools::{ReportSection, ReportStatus, ToolParser, ToolReport};

pub struct FlamegraphParser;

impl ToolParser for FlamegraphParser {
    fn parse(&self, _stdout: &str, _stderr: &str, exit_code: i32) -> ToolReport {
        let status = if exit_code != 0 {
            ReportStatus::Error
        } else {
            ReportStatus::Ok
        };
        ToolReport {
            status,
            sections: vec![ReportSection {
                title: "Flamegraph".to_string(),
                content: "| Field | Value |\n|-------|-------|\n| Output | flamegraph.svg |"
                    .to_string(),
            }],
        }
    }
}
