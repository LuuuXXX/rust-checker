use crate::tools::{ReportSection, ReportStatus, ToolParser, ToolReport};

pub struct BloatParser;

impl ToolParser for BloatParser {
    fn parse(&self, stdout: &str, stderr: &str, exit_code: i32) -> ToolReport {
        let combined = format!("{}\n{}", stdout, stderr);
        let status = if exit_code != 0 {
            ReportStatus::Error
        } else {
            ReportStatus::Ok
        };

        let mut rows =
            vec!["| Symbol | Size | Crate |\n|--------|------|-------|".to_string()];
        for line in combined.lines().take(20) {
            if line.contains('%') {
                rows.push(format!("| - | - | {} |", line.trim()));
            }
        }

        ToolReport {
            status,
            sections: vec![ReportSection {
                title: "Binary Bloat".to_string(),
                content: rows.join("\n"),
            }],
        }
    }
}
