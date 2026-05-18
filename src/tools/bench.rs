use crate::report::{ToolReport, ToolStatus};

pub fn parse(stdout: &str, stderr: &str, exit_code: i32, command: &str) -> ToolReport {
    let combined = format!("{}\n{}", stdout, stderr);

    // Extract benchmark results: lines like "test bench_foo ... bench: 1,234 ns/iter (+/- 56)"
    let bench_results: Vec<&str> = combined
        .lines()
        .filter(|l| l.contains("bench:") || l.contains("ns/iter"))
        .collect();

    let bench_count = bench_results.len();

    let status = if exit_code != 0 {
        ToolStatus::Error
    } else {
        ToolStatus::Ok
    };

    let summary = if exit_code != 0 {
        "基准测试失败".to_string()
    } else if bench_count > 0 {
        format!("完成 {} 个基准测试", bench_count)
    } else {
        "基准测试完成".to_string()
    };

    let bench_section = if bench_results.is_empty() {
        String::new()
    } else {
        format!(
            "\n## 基准测试结果\n\n```\n{}\n```\n",
            bench_results.join("\n")
        )
    };

    let markdown_content = format!(
        "# Bench\n\n**命令**: `{command}`\n\n**状态**: {}\n\n**摘要**: {}{}\n\n## 完整输出\n\n```\n{}\n```\n",
        if exit_code == 0 { "✅ 成功" } else { "❌ 失败" },
        summary,
        bench_section,
        combined.trim()
    );

    ToolReport {
        tool_name: "bench".to_string(),
        status,
        summary,
        output_path: "perf/bench.md".to_string(),
        markdown_content,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::ToolStatus;

    #[test]
    fn test_bench_ok() {
        let stdout =
            "test bench_add ... bench:          10 ns/iter (+/- 1)\ntest bench_mul ... bench:          20 ns/iter (+/- 2)";
        let r = parse(stdout, "", 0, "cargo bench");
        assert_eq!(r.status, ToolStatus::Ok);
        assert!(r.summary.contains("2"));
    }

    #[test]
    fn test_bench_fail() {
        let r = parse("", "error: bench failed", 1, "cargo bench");
        assert_eq!(r.status, ToolStatus::Error);
    }

    #[test]
    fn test_bench_no_results() {
        let r = parse("running 0 benchmarks", "", 0, "cargo bench");
        assert_eq!(r.status, ToolStatus::Ok);
    }
}
