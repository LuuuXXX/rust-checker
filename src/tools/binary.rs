use crate::tools::{ReportSection, ReportStatus, ToolParser, ToolReport};

pub struct BinaryParser;

impl ToolParser for BinaryParser {
    fn parse(&self, stdout: &str, stderr: &str, exit_code: i32) -> ToolReport {
        let combined = format!("{}\n{}", stdout, stderr);
        let status = if exit_code != 0 {
            ReportStatus::Error
        } else {
            ReportStatus::Ok
        };

        let mut rows =
            vec!["| Binary | Size | Type |\n|--------|------|------|".to_string()];
        for line in combined.lines() {
            if line.contains("target/")
                && (line.contains("release") || line.contains("debug"))
            {
                rows.push(format!("| {} | - | binary |", line.trim()));
            }
        }

        ToolReport {
            status,
            sections: vec![ReportSection {
                title: "Binary Artifacts".to_string(),
                content: rows.join("\n"),
            }],
        }
    }
}
