use crate::tools::{ReportSection, ReportStatus, ToolParser, ToolReport};

pub struct UdepsParser;

impl ToolParser for UdepsParser {
    fn parse(&self, stdout: &str, stderr: &str, exit_code: i32) -> ToolReport {
        let combined = format!("{}\n{}", stdout, stderr);
        let mut unused: Vec<String> = Vec::new();

        let mut in_unused = false;
        for line in combined.lines() {
            if line.contains("unused dependencies") {
                in_unused = true;
            }
            if in_unused && line.trim().starts_with('`') {
                let name = line.trim().trim_matches('`').to_string();
                unused.push(name);
            }
        }

        let status = if !unused.is_empty() {
            ReportStatus::Warn
        } else if exit_code != 0 {
            ReportStatus::Error
        } else {
            ReportStatus::Ok
        };

        let mut rows = vec!["| Crate | Reason |\n|-------|--------|".to_string()];
        for u in &unused {
            rows.push(format!("| {} | unused |", u));
        }

        ToolReport {
            status,
            sections: vec![ReportSection {
                title: "Unused Dependencies".to_string(),
                content: rows.join("\n"),
            }],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::ToolParser;

    #[test]
    fn test_udeps_clean() {
        let stdout = "All dependencies are used.\n";
        let report = UdepsParser.parse(stdout, "", 0);
        assert_eq!(report.status, ReportStatus::Ok);
    }
}
