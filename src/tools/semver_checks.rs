use crate::tools::{ReportSection, ReportStatus, ToolParser, ToolReport};

pub struct SemverChecksParser;

impl ToolParser for SemverChecksParser {
    fn parse(&self, stdout: &str, stderr: &str, exit_code: i32) -> ToolReport {
        let combined = format!("{}\n{}", stdout, stderr);
        let breaking: Vec<&str> = combined
            .lines()
            .filter(|l| l.contains("breaking") || l.contains("major"))
            .collect();

        let status = if exit_code != 0 || !breaking.is_empty() {
            ReportStatus::Error
        } else {
            ReportStatus::Ok
        };

        let mut rows = vec!["| Change | Severity |\n|--------|---------|".to_string()];
        for line in &breaking {
            rows.push(format!("| {} | breaking |", line.trim()));
        }

        ToolReport {
            status,
            sections: vec![ReportSection {
                title: "Semver Compatibility".to_string(),
                content: rows.join("\n"),
            }],
        }
    }
}
