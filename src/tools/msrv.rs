use crate::report::{ToolReport, ToolStatus};

pub fn parse(stdout: &str, stderr: &str, exit_code: i32, command: &str) -> ToolReport {
    let combined = format!("{}\n{}", stdout, stderr);

    // Look for MSRV version in output like "MSRV: 1.65.0" or "Minimum Supported Rust Version: 1.65.0"
    let mut msrv_version: Option<String> = None;
    for line in combined.lines() {
        let lower = line.to_lowercase();
        if lower.contains("msrv") || lower.contains("minimum supported rust version") {
            // Try to extract version number (e.g. 1.65.0 or `1.65.0`)
            for part in line.split_whitespace() {
                let trimmed = part.trim_matches(|c: char| !c.is_alphanumeric() && c != '.');
                if trimmed
                    .chars()
                    .next()
                    .map(|c| c.is_ascii_digit())
                    .unwrap_or(false)
                    && trimmed.contains('.')
                {
                    msrv_version = Some(trimmed.to_string());
                    break;
                }
            }
        }
    }

    let status = if exit_code != 0 {
        ToolStatus::Error
    } else {
        ToolStatus::Ok
    };

    let summary = match (exit_code, &msrv_version) {
        (0, Some(v)) => format!("MSRV: {}", v),
        (0, None) => "MSRV 检查完成".to_string(),
        _ => "MSRV 检查失败".to_string(),
    };

    let markdown_content = format!(
        "# MSRV\n\n**命令**: `{command}`\n\n**状态**: {}\n\n**摘要**: {}\n\n## 输出\n\n```\n{}\n```\n",
        if exit_code == 0 { "✅ 成功" } else { "❌ 失败" },
        summary,
        combined.trim()
    );

    ToolReport {
        tool_name: "msrv".to_string(),
        status,
        summary,
        output_path: "compat/msrv.md".to_string(),
        markdown_content,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::ToolStatus;

    #[test]
    fn test_msrv_found() {
        let stdout = "Minimum Supported Rust Version: 1.65.0";
        let r = parse(stdout, "", 0, "cargo msrv");
        assert_eq!(r.status, ToolStatus::Ok);
        assert!(r.summary.contains("1.65.0"));
    }

    #[test]
    fn test_msrv_found_backtick_wrapped() {
        // cargo msrv ≥0.15 wraps the version in backticks: "MSRV is `1.65.0`"
        let stdout = "MSRV is `1.65.0`";
        let r = parse(stdout, "", 0, "cargo msrv");
        assert_eq!(r.status, ToolStatus::Ok);
        assert!(
            r.summary.contains("1.65.0"),
            "backtick-wrapped version must be extracted: {}",
            r.summary
        );
    }

    #[test]
    fn test_msrv_not_found() {
        let r = parse("", "", 0, "cargo msrv");
        assert_eq!(r.status, ToolStatus::Ok);
    }

    #[test]
    fn test_msrv_fail() {
        let r = parse("", "error", 1, "cargo msrv");
        assert_eq!(r.status, ToolStatus::Error);
    }
}
