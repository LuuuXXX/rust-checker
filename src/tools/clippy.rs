use crate::tools::{ReportSection, ReportStatus, ToolParser, ToolReport};

pub struct ClippyParser;

impl ToolParser for ClippyParser {
    fn parse(&self, stdout: &str, stderr: &str, exit_code: i32) -> ToolReport {
        let combined = format!("{}\n{}", stdout, stderr);
        let warning_lines: Vec<&str> = combined
            .lines()
            .filter(|l| l.trim_start().starts_with("warning:"))
            .collect();
        let error_lines: Vec<&str> = combined
            .lines()
            .filter(|l| l.trim_start().starts_with("error:"))
            .collect();

        let status = if exit_code != 0 {
            ReportStatus::Error
        } else if !warning_lines.is_empty() {
            ReportStatus::Warn
        } else {
            ReportStatus::Ok
        };

        let summary = format!(
            "| Field | Count |\n|-------|-------|\n| Warnings | {} |\n| Errors | {} |",
            warning_lines.len(),
            error_lines.len()
        );

        let mut issue_rows = vec![
            "| Level | Rule | File | Line | Message |\n|-------|------|------|------|---------|"
                .to_string(),
        ];
        for line in warning_lines.iter().chain(error_lines.iter()) {
            issue_rows.push(format!("| warn | - | - | - | {} |", line.trim()));
        }

        ToolReport {
            status,
            sections: vec![
                ReportSection {
                    title: "Clippy Summary".to_string(),
                    content: summary,
                },
                ReportSection {
                    title: "Clippy Issues".to_string(),
                    content: issue_rows.join("\n"),
                },
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::ToolParser;

    #[test]
    fn test_clippy_clean() {
        let report = ClippyParser.parse("", "", 0);
        assert_eq!(report.status, ReportStatus::Ok);
    }

    #[test]
    fn test_clippy_with_warnings() {
        let stderr = "warning: unused variable `x`\n --> src/main.rs:5:9\n";
        let report = ClippyParser.parse("", stderr, 0);
        assert_eq!(report.status, ReportStatus::Warn);
    }
}
