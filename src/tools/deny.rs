use crate::tools::{ReportSection, ReportStatus, ToolParser, ToolReport};

pub struct DenyParser;

impl ToolParser for DenyParser {
    fn parse(&self, stdout: &str, stderr: &str, exit_code: i32) -> ToolReport {
        let combined = format!("{}\n{}", stdout, stderr);
        let issue_lines: Vec<&str> = combined
            .lines()
            .filter(|l| l.contains("denied") || l.contains("error["))
            .collect();

        let status = if exit_code != 0 {
            ReportStatus::Error
        } else {
            ReportStatus::Ok
        };

        let summary = format!(
            "| Check | Result |\n|-------|--------|\n| Bans | {} |\n| Licenses | {} |\n| Advisories | {} |\n| Sources | {} |",
            if issue_lines.iter().any(|l| l.contains("ban")) { "❌" } else { "✅" },
            if issue_lines.iter().any(|l| l.contains("licens")) { "❌" } else { "✅" },
            if issue_lines.iter().any(|l| l.contains("advisor")) { "❌" } else { "✅" },
            if issue_lines.iter().any(|l| l.contains("source")) { "❌" } else { "✅" },
        );

        let mut issue_rows =
            vec!["| Type | Crate | Reason |\n|------|-------|--------|".to_string()];
        for line in &issue_lines {
            issue_rows.push(format!("| deny | - | {} |", line.trim()));
        }

        ToolReport {
            status,
            sections: vec![
                ReportSection {
                    title: "Deny Summary".to_string(),
                    content: summary,
                },
                ReportSection {
                    title: "Deny Issues".to_string(),
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
    fn test_deny_ok() {
        let report = DenyParser.parse("", "", 0);
        assert_eq!(report.status, ReportStatus::Ok);
    }
}
