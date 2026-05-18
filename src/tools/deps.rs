use crate::tools::{ReportSection, ReportStatus, ToolParser, ToolReport};

pub struct DepsParser;

impl ToolParser for DepsParser {
    fn parse(&self, stdout: &str, stderr: &str, exit_code: i32) -> ToolReport {
        let lines: Vec<&str> = stdout.lines().collect();
        let total = lines.len();
        let direct = lines
            .iter()
            .filter(|l| !l.starts_with(' ') && !l.starts_with('|'))
            .count();
        let transitive = total.saturating_sub(direct);

        let status = if exit_code != 0 {
            ReportStatus::Error
        } else {
            ReportStatus::Ok
        };

        let summary = format!(
            "| Metric | Count |\n|--------|-------|\n| Total | {} |\n| Direct | {} |\n| Transitive | {} |",
            total, direct, transitive
        );

        let mut dep_rows =
            vec!["| Name | Version | Depth |\n|------|---------|-------|".to_string()];
        for (i, line) in lines.iter().enumerate().take(20) {
            let depth = line
                .chars()
                .take_while(|c| *c == ' ' || *c == '|')
                .count()
                / 4;
            let name =
                line.trim_start_matches(|c: char| c == ' ' || c == '|' || c == '-');
            dep_rows.push(format!("| {} | - | {} |", name, depth));
            if i >= 19 {
                break;
            }
        }

        // suppress unused stderr warning
        let _ = stderr;

        ToolReport {
            status,
            sections: vec![
                ReportSection {
                    title: "Dependency Summary".to_string(),
                    content: summary,
                },
                ReportSection {
                    title: "All Dependencies".to_string(),
                    content: dep_rows.join("\n"),
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
    fn test_deps_parse() {
        let stdout =
            "my-crate v0.1.0\n├── serde v1.0.0\n│   └── serde_derive v1.0.0\n└── tokio v1.0.0\n";
        let report = DepsParser.parse(stdout, "", 0);
        assert_eq!(report.status, ReportStatus::Ok);
    }
}
