use crate::tools::{ReportSection, ReportStatus, ToolParser, ToolReport};

pub struct DocParser;

impl ToolParser for DocParser {
    fn parse(&self, stdout: &str, stderr: &str, exit_code: i32) -> ToolReport {
        let combined = format!("{}\n{}", stdout, stderr);
        let warning_lines: Vec<&str> = combined
            .lines()
            .filter(|l| l.contains("warning:"))
            .collect();

        let status = if exit_code != 0 {
            ReportStatus::Error
        } else if !warning_lines.is_empty() {
            ReportStatus::Warn
        } else {
            ReportStatus::Ok
        };

        let summary = format!(
            "| Field | Value |\n|-------|-------|\n| Status | {} |\n| Warnings | {} |",
            if exit_code == 0 {
                "✅ Success"
            } else {
                "❌ Failed"
            },
            warning_lines.len()
        );

        let mut warn_rows =
            vec!["| File | Line | Message |\n|------|------|---------|".to_string()];
        for line in &warning_lines {
            warn_rows.push(format!("| - | - | {} |", line.trim()));
        }

        ToolReport {
            status,
            sections: vec![
                ReportSection {
                    title: "Doc Status".to_string(),
                    content: summary,
                },
                ReportSection {
                    title: "Doc Warnings".to_string(),
                    content: warn_rows.join("\n"),
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
    fn test_doc_ok() {
        let stdout =
            "   Documenting my-crate v0.1.0\n    Finished dev target(s) in 2.0s\n";
        let report = DocParser.parse(stdout, "", 0);
        assert_eq!(report.status, ReportStatus::Ok);
    }
}
