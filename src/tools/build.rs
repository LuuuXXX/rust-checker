use crate::tools::{ReportSection, ReportStatus, ToolParser, ToolReport};

pub struct BuildParser;

impl ToolParser for BuildParser {
    fn parse(&self, stdout: &str, stderr: &str, exit_code: i32) -> ToolReport {
        let combined = format!("{}\n{}", stdout, stderr);
        let warning_lines: Vec<&str> = combined
            .lines()
            .filter(|l| l.contains("warning[") || l.starts_with("warning:"))
            .collect();
        let error_lines: Vec<&str> = combined
            .lines()
            .filter(|l| {
                l.contains("error[") || (l.starts_with("error") && !l.contains("aborting"))
            })
            .collect();

        let status = if exit_code != 0 {
            ReportStatus::Error
        } else if !warning_lines.is_empty() {
            ReportStatus::Warn
        } else {
            ReportStatus::Ok
        };

        let build_status = format!(
            "| Field | Value |\n|-------|-------|\n| Result | {} |\n| Warnings | {} |\n| Errors | {} |",
            if exit_code == 0 { "✅ Success" } else { "❌ Failed" },
            warning_lines.len(),
            error_lines.len()
        );

        let mut warning_rows =
            vec!["| File | Line | Message |\n|------|------|---------|".to_string()];
        for line in &warning_lines {
            warning_rows.push(format!("| - | - | {} |", line.trim()));
        }

        let mut error_rows =
            vec!["| File | Line | Message |\n|------|------|---------|".to_string()];
        for line in &error_lines {
            error_rows.push(format!("| - | - | {} |", line.trim()));
        }

        ToolReport {
            status,
            sections: vec![
                ReportSection {
                    title: "Build Status".to_string(),
                    content: build_status,
                },
                ReportSection {
                    title: "Warning Summary".to_string(),
                    content: warning_rows.join("\n"),
                },
                ReportSection {
                    title: "Error Summary".to_string(),
                    content: error_rows.join("\n"),
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
    fn test_build_success() {
        let stdout = "   Compiling my-crate v0.1.0\n    Finished dev [unoptimized + debuginfo] target(s) in 1.23s\n";
        let stderr = "";
        let report = BuildParser.parse(stdout, stderr, 0);
        assert_eq!(report.status, ReportStatus::Ok);
    }

    #[test]
    fn test_build_with_warnings() {
        let stderr = "warning: unused variable `x`\n --> src/main.rs:5:9\n";
        let report = BuildParser.parse("", stderr, 0);
        assert_eq!(report.status, ReportStatus::Warn);
    }

    #[test]
    fn test_build_failure() {
        let stderr =
            "error[E0425]: cannot find value `foo` in this scope\n --> src/main.rs:3:5\n";
        let report = BuildParser.parse("", stderr, 1);
        assert_eq!(report.status, ReportStatus::Error);
    }
}
