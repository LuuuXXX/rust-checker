use crate::tools::{ReportSection, ReportStatus, ToolParser, ToolReport};

pub struct BenchParser;

impl ToolParser for BenchParser {
    fn parse(&self, stdout: &str, stderr: &str, exit_code: i32) -> ToolReport {
        let combined = format!("{}\n{}", stdout, stderr);
        let bench_lines: Vec<&str> = combined
            .lines()
            .filter(|l| l.contains("bench:") || l.contains("ns/iter"))
            .collect();

        let status = if exit_code != 0 {
            ReportStatus::Error
        } else {
            ReportStatus::Ok
        };

        let mut rows = vec!["| Benchmark | Time |\n|-----------|------|".to_string()];
        for line in &bench_lines {
            rows.push(format!("| - | {} |", line.trim()));
        }

        ToolReport {
            status,
            sections: vec![ReportSection {
                title: "Benchmark Results".to_string(),
                content: rows.join("\n"),
            }],
        }
    }
}
