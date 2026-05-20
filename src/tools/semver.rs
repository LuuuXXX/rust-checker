use crate::report::{ToolReport, ToolStatus};

pub fn parse(stdout: &str, stderr: &str, exit_code: i32, command: &str) -> ToolReport {
    let combined = format!("{}\n{}", stdout, stderr);

    // Look for FAILED or passed in output
    let has_failed = combined.lines().any(|l| {
        l.contains("FAILED") || (l.contains("semver violations") && !l.contains("no semver"))
    });
    let has_passed = combined
        .lines()
        .any(|l| l.contains("passed") || l.contains("no semver"));

    let status = if exit_code != 0 || has_failed {
        ToolStatus::Error
    } else {
        ToolStatus::Ok
    };

    // Count violations:
    // 1. Primary "FAILED" lines (one per API break in verbose output).
    // 2. Summary line "N semver violations found" when individual FAILED lines are absent
    //    (some versions / invocations only emit the summary).
    let failed_line_count = combined.lines().filter(|l| l.contains("FAILED")).count();
    let summary_count: usize = combined
        .lines()
        .find(|l| l.contains("semver violations") && !l.contains("no semver"))
        .and_then(|l| l.split_whitespace().find_map(|p| p.parse().ok()))
        .unwrap_or(0);
    let violation_count = if failed_line_count > 0 {
        failed_line_count
    } else {
        summary_count
    };

    let summary = if has_failed || exit_code != 0 {
        if violation_count > 0 {
            format!("发现 {} 处 semver 违规", violation_count)
        } else {
            "semver 检查失败".to_string()
        }
    } else if has_passed {
        "semver 兼容性检查通过".to_string()
    } else {
        "semver 检查完成".to_string()
    };

    let markdown_content = format!(
        "# Semver\n\n**命令**: `{command}`\n\n**状态**: {}\n\n**摘要**: {}\n\n## 输出\n\n```\n{}\n```\n",
        if status == ToolStatus::Ok { "✅ 兼容" } else { "❌ 违规" },
        summary,
        combined.trim()
    );

    ToolReport {
        tool_name: "semver".to_string(),
        status,
        summary,
        output_path: "compat/semver.md".to_string(),
        markdown_content,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::ToolStatus;

    #[test]
    fn test_semver_ok() {
        let stdout = "Checking foo v0.1.0\nno semver violations found\n1 passed";
        let r = parse(stdout, "", 0, "cargo semver-checks");
        assert_eq!(r.status, ToolStatus::Ok);
    }

    #[test]
    fn test_semver_failed() {
        let stdout = "FAILED: removed public function `bar`";
        let r = parse(stdout, "", 1, "cargo semver-checks");
        assert_eq!(r.status, ToolStatus::Error);
    }

    #[test]
    fn test_semver_failed_multiline_context_not_inflated() {
        // A single violation spans multiple lines in real semver-checks output.
        // Context lines containing "violation" must not inflate the count.
        let stdout = "FAILED: removed public function `bar`\n  \
            This constitutes a semver violation: public items must not be removed.\n  \
            API: src/lib.rs:42";
        let r = parse(stdout, "", 1, "cargo semver-checks");
        assert_eq!(r.status, ToolStatus::Error);
        assert!(
            r.summary.contains("1"),
            "single violation must report 1, not inflated: {}",
            r.summary
        );
        assert!(
            !r.summary.contains('2') && !r.summary.contains('3'),
            "violation count must not be inflated by context lines: {}",
            r.summary
        );
    }

    #[test]
    fn test_semver_empty() {
        let r = parse("", "", 0, "cargo semver-checks");
        assert_eq!(r.status, ToolStatus::Ok);
    }

    #[test]
    fn test_semver_summary_line_count() {
        // Some versions of cargo semver-checks only emit a summary line like
        // "3 semver violations found" without individual FAILED: lines.
        let stdout = "Checking foo v0.1.0\n3 semver violations found";
        let r = parse(stdout, "", 1, "cargo semver-checks");
        assert_eq!(r.status, ToolStatus::Error);
        assert!(
            r.summary.contains("3"),
            "violation count from summary line must be reported: {}",
            r.summary
        );
    }
}
