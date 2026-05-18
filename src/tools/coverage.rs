use crate::tools::{ReportSection, ReportStatus, ToolParser, ToolReport};

pub struct CoverageParser;

impl ToolParser for CoverageParser {
    fn parse(&self, stdout: &str, stderr: &str, exit_code: i32) -> ToolReport {
        let combined = format!("{}\n{}", stdout, stderr);

        let mut line_pct = "N/A".to_string();
        let branch_pct = "N/A".to_string();
        let func_pct = "N/A".to_string();

        for line in combined.lines() {
            // llvm-cov output: "TOTAL    ... 75.00%"
            if line.starts_with("TOTAL") || line.contains("Total") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    line_pct = parts[parts.len() - 1].to_string();
                }
            }
        }

        let status = if exit_code != 0 {
            ReportStatus::Error
        } else {
            ReportStatus::Ok
        };

        let summary = format!(
            "| Metric | Value |\n|--------|-------|\n| Line Coverage | {} |\n| Branch Coverage | {} |\n| Function Coverage | {} |",
            line_pct, branch_pct, func_pct
        );

        ToolReport {
            status,
            sections: vec![ReportSection {
                title: "Coverage Summary".to_string(),
                content: summary,
            }],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::ToolParser;

    #[test]
    fn test_coverage_parse() {
        let stdout = "Filename         Regions  Missed  Cover  Functions  Missed  Cover  Lines  Missed  Cover\n\
                      TOTAL            10       2       80.00%  5          1       80.00%  50     5       90.00%\n";
        let report = CoverageParser.parse(stdout, "", 0);
        assert_eq!(report.status, ReportStatus::Ok);
    }
}
