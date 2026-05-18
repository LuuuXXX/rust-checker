use crate::tools::{ReportSection, ReportStatus, ToolParser, ToolReport};

pub struct FmtParser;

impl ToolParser for FmtParser {
    fn parse(&self, stdout: &str, stderr: &str, exit_code: i32) -> ToolReport {
        let combined = format!("{}\n{}", stdout, stderr);
        let needs_fmt: Vec<&str> = combined
            .lines()
            .filter(|l| l.ends_with(".rs") || l.contains("Diff in"))
            .collect();

        let status = if exit_code != 0 {
            ReportStatus::Warn
        } else {
            ReportStatus::Ok
        };

        let content = format!(
            "| Field | Value |\n|-------|-------|\n| Status | {} |\n| Files needing format | {} |",
            if exit_code == 0 {
                "✅ Formatted"
            } else {
                "⚠️ Needs formatting"
            },
            needs_fmt.len()
        );

        ToolReport {
            status,
            sections: vec![ReportSection {
                title: "Format Status".to_string(),
                content,
            }],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::ToolParser;

    #[test]
    fn test_fmt_ok() {
        let report = FmtParser.parse("", "", 0);
        assert_eq!(report.status, ReportStatus::Ok);
    }

    #[test]
    fn test_fmt_needs_format() {
        let report = FmtParser.parse("", "Diff in src/main.rs at line 5:\n", 1);
        assert_eq!(report.status, ReportStatus::Warn);
    }
}
