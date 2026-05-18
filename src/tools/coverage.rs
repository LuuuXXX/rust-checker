use crate::report::{ToolReport, ToolStatus};

pub fn parse(stdout: &str, stderr: &str, exit_code: i32, command: &str) -> ToolReport {
    let combined = format!("{}\n{}", stdout, stderr);

    // Look for "TOTAL" line with percentage, e.g. "TOTAL  80.00%  120/150"
    let mut coverage_pct: Option<f64> = None;
    for line in combined.lines() {
        let upper = line.to_uppercase();
        if upper.contains("TOTAL") {
            // Try to find a percentage value
            for part in line.split_whitespace() {
                if part.ends_with('%') {
                    if let Ok(pct) = part.trim_end_matches('%').parse::<f64>() {
                        coverage_pct = Some(pct);
                        break;
                    }
                }
            }
        }
    }

    let status = if exit_code != 0 {
        ToolStatus::Error
    } else if let Some(pct) = coverage_pct {
        if pct < 60.0 {
            ToolStatus::Warn
        } else {
            ToolStatus::Ok
        }
    } else {
        ToolStatus::Ok
    };

    let summary = match coverage_pct {
        Some(pct) => format!("覆盖率: {:.1}%", pct),
        None if exit_code == 0 => "覆盖率报告已生成".to_string(),
        None => "覆盖率检查失败".to_string(),
    };

    let markdown_content = format!(
        "# Coverage\n\n**命令**: `{command}`\n\n**状态**: {}\n\n**摘要**: {}\n\n## 输出\n\n```\n{}\n```\n",
        if exit_code == 0 { "✅ 成功" } else { "❌ 失败" },
        summary,
        combined.trim()
    );

    ToolReport {
        tool_name: "coverage".to_string(),
        status,
        summary,
        output_path: "quality/coverage.md".to_string(),
        markdown_content,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::ToolStatus;

    #[test]
    fn test_parse_coverage_ok() {
        let stdout = "Filename  Regions  Missed  Cover\nfoo.rs    100      10      90.00%\nTOTAL     100      10      90.00%";
        let r = parse(stdout, "", 0, "cargo llvm-cov");
        assert_eq!(r.status, ToolStatus::Ok);
        assert!(r.summary.contains("90.0%"));
    }

    #[test]
    fn test_parse_coverage_warn() {
        let stdout = "TOTAL  100  50  50.00%";
        let r = parse(stdout, "", 0, "cargo llvm-cov");
        assert_eq!(r.status, ToolStatus::Warn);
        assert!(r.summary.contains("50.0%"));
    }

    #[test]
    fn test_parse_no_data() {
        let r = parse("", "", 0, "cargo llvm-cov");
        assert_eq!(r.status, ToolStatus::Ok);
    }
}
