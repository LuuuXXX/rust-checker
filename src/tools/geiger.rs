use crate::tools::{ReportSection, ReportStatus, ToolParser, ToolReport};

pub struct GeigerParser;

impl ToolParser for GeigerParser {
    fn parse(&self, stdout: &str, stderr: &str, exit_code: i32) -> ToolReport {
        let combined = format!("{}\n{}", stdout, stderr);

        let mut total_unsafe = 0usize;
        let mut crate_rows = vec![
            "| Crate | Unsafe Fns | Unsafe Exprs |\n|-------|-----------|-------------|".to_string(),
        ];

        for line in combined.lines() {
            if line.contains('!') && (line.contains("fn") || line.contains("expr")) {
                total_unsafe += 1;
                crate_rows.push(format!("| - | - | {} |", line.trim()));
            }
        }

        let status = if exit_code != 0 {
            ReportStatus::Error
        } else if total_unsafe > 0 {
            ReportStatus::Warn
        } else {
            ReportStatus::Ok
        };

        let summary = format!(
            "| Field | Value |\n|-------|-------|\n| Total Unsafe | {} |",
            total_unsafe
        );

        ToolReport {
            status,
            sections: vec![
                ReportSection {
                    title: "Geiger Summary".to_string(),
                    content: summary,
                },
                ReportSection {
                    title: "Unsafe Usage".to_string(),
                    content: crate_rows.join("\n"),
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
    fn test_geiger_clean() {
        let report = GeigerParser.parse("", "", 0);
        assert_eq!(report.status, ReportStatus::Ok);
    }
}
