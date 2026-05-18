use crate::tools::{ReportSection, ReportStatus, ToolParser, ToolReport};

pub struct TestParser;

impl ToolParser for TestParser {
    fn parse(&self, stdout: &str, stderr: &str, exit_code: i32) -> ToolReport {
        let combined = format!("{}\n{}", stdout, stderr);

        let mut passed = 0usize;
        let mut failed = 0usize;
        let mut ignored = 0usize;
        let mut failed_tests: Vec<String> = Vec::new();

        for line in combined.lines() {
            if line.contains("test result:") {
                // "test result: ok. 5 passed; 0 failed; 0 ignored;"
                if let Some(p) = extract_count(line, "passed") {
                    passed = p;
                }
                if let Some(f) = extract_count(line, "failed") {
                    failed = f;
                }
                if let Some(i) = extract_count(line, "ignored") {
                    ignored = i;
                }
            }
            if line.starts_with("FAILED") || line.contains("... FAILED") {
                failed_tests.push(line.trim().to_string());
            }
        }

        let total = passed + failed + ignored;
        let status = if exit_code != 0 || failed > 0 {
            ReportStatus::Error
        } else {
            ReportStatus::Ok
        };

        let summary = format!(
            "| Field | Value |\n|-------|-------|\n| Total | {} |\n| Passed | {} |\n| Failed | {} |\n| Ignored | {} |",
            total, passed, failed, ignored
        );

        let mut failed_rows = vec!["| Test | Error |\n|------|-------|".to_string()];
        for t in &failed_tests {
            failed_rows.push(format!("| {} | - |", t));
        }

        ToolReport {
            status,
            sections: vec![
                ReportSection {
                    title: "Test Summary".to_string(),
                    content: summary,
                },
                ReportSection {
                    title: "Failed Tests".to_string(),
                    content: failed_rows.join("\n"),
                },
            ],
        }
    }
}

fn extract_count(line: &str, keyword: &str) -> Option<usize> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    for (i, part) in parts.iter().enumerate() {
        if *part == keyword || part.starts_with(keyword) {
            if i > 0 {
                return parts[i - 1].trim_end_matches(';').parse().ok();
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::ToolParser;

    #[test]
    fn test_parse_test_results() {
        let stdout =
            "test foo ... ok\ntest bar ... ok\ntest result: ok. 2 passed; 0 failed; 0 ignored;\n";
        let report = TestParser.parse(stdout, "", 0);
        assert_eq!(report.status, ReportStatus::Ok);
    }

    #[test]
    fn test_parse_test_failures() {
        let stdout =
            "test foo ... FAILED\ntest result: FAILED. 0 passed; 1 failed; 0 ignored;\n";
        let report = TestParser.parse(stdout, "", 1);
        assert_eq!(report.status, ReportStatus::Error);
    }
}
