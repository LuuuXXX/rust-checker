use crate::tools::{ReportSection, ReportStatus, ToolParser, ToolReport};

pub struct AuditParser;

impl ToolParser for AuditParser {
    fn parse(&self, stdout: &str, stderr: &str, exit_code: i32) -> ToolReport {
        let combined = format!("{}\n{}", stdout, stderr);

        let vuln_lines: Vec<&str> = combined
            .lines()
            .filter(|l| l.contains("Crate:") || l.contains("Advisory:") || l.contains("Severity:"))
            .collect();

        let status = if exit_code != 0 {
            ReportStatus::Error
        } else if combined.contains("warning") {
            ReportStatus::Warn
        } else {
            ReportStatus::Ok
        };

        let vulns = vuln_lines.len() / 3; // rough count

        let summary = format!(
            "| Field | Value |\n|-------|-------|\n| Vulnerabilities | {} |\n| Warnings | 0 |",
            vulns
        );

        let mut vuln_rows = vec![
            "| Crate | Version | Advisory | Severity |\n|-------|---------|----------|---------|"
                .to_string(),
        ];
        let mut i = 0;
        while i + 2 < vuln_lines.len() {
            vuln_rows.push(format!(
                "| {} | - | {} | {} |",
                vuln_lines[i].trim(),
                vuln_lines[i + 1].trim(),
                vuln_lines[i + 2].trim()
            ));
            i += 3;
        }

        ToolReport {
            status,
            sections: vec![
                ReportSection {
                    title: "Audit Summary".to_string(),
                    content: summary,
                },
                ReportSection {
                    title: "Vulnerabilities".to_string(),
                    content: vuln_rows.join("\n"),
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
    fn test_audit_clean() {
        let stdout = "    Fetching advisory database from `https://github.com/RustSec/advisory-db.git`\n\
                      0 vulnerabilities found\n";
        let report = AuditParser.parse(stdout, "", 0);
        assert_eq!(report.status, ReportStatus::Ok);
    }

    #[test]
    fn test_audit_with_vulns() {
        let stdout = "Crate: openssl\nAdvisory: RUSTSEC-2021-0001\nSeverity: high\n";
        let report = AuditParser.parse(stdout, "", 1);
        assert_eq!(report.status, ReportStatus::Error);
    }
}
