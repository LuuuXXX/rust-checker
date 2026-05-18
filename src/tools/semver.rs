use crate::report::{ToolReport, ToolStatus};

pub fn parse(stdout: &str, stderr: &str, exit_code: i32, command: &str) -> ToolReport {
    let combined = format!("{}\n{}", stdout, stderr);

    // Look for FAILED or passed in output
    let has_failed = combined
        .lines()
        .any(|l| l.contains("FAILED") || (l.contains("semver violations") && !l.contains("no semver")));
    let has_passed = combined
        .lines()
        .any(|l| l.contains("passed") || l.contains("no semver"));

    let status = if exit_code != 0 || has_failed {
        ToolStatus::Error
    } else {
        ToolStatus::Ok
    };

    // Count violation count if present
    let violation_count = combined
        .lines()
        .filter(|l| l.contains("FAILED") || l.contains("violation"))
        .count();

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
    fn test_semver_empty() {
        let r = parse("", "", 0, "cargo semver-checks");
        assert_eq!(r.status, ToolStatus::Ok);
    }
}
