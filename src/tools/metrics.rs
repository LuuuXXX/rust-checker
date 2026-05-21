use crate::report::{ToolReport, ToolStatus};

pub fn parse(stdout: &str, stderr: &str, exit_code: i32, command: &str) -> ToolReport {
    let combined = format!("{}\n{}", stdout, stderr);

    let status = if exit_code != 0 {
        ToolStatus::Error
    } else {
        ToolStatus::Ok
    };

    let summary = if exit_code != 0 {
        "指标收集失败".to_string()
    } else {
        "unsafe 代码安全性指标已收集".to_string()
    };

    let markdown_content = format!(
        "# Metrics\n\n**命令**: `{command}`\n\n**状态**: {}\n\n**摘要**: {}\n\n## 输出\n\n```\n{}\n```\n",
        if exit_code == 0 { "✅ 成功" } else { "❌ 失败" },
        summary,
        combined.trim()
    );

    ToolReport {
        tool_name: "metrics".to_string(),
        status,
        summary,
        output_path: "perf/metrics.md".to_string(),
        markdown_content,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::ToolStatus;

    #[test]
    fn test_metrics_ok() {
        let r = parse(
            "src/main.rs: 100 lines\nsrc/lib.rs: 200 lines",
            "",
            0,
            "cargo geiger --output-format Ratio",
        );
        assert_eq!(r.status, ToolStatus::Ok);
        assert!(r.summary.contains("收集"));
    }

    #[test]
    fn test_metrics_fail() {
        let r = parse("", "error", 1, "cargo geiger --output-format Ratio");
        assert_eq!(r.status, ToolStatus::Error);
    }
}
