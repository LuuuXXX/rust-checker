use crate::report::{ToolReport, ToolStatus};

pub fn parse(stdout: &str, stderr: &str, exit_code: i32, command: &str) -> ToolReport {
    let combined = format!("{}\n{}", stdout, stderr);

    // cargo deny outputs "error[" lines for denies
    let error_count = combined
        .lines()
        .filter(|l| l.contains("error[") || (l.starts_with("error") && !l.contains("warning")))
        .count();

    let warning_count = combined.lines().filter(|l| l.contains("warning[")).count();

    let status = if error_count > 0 || exit_code != 0 {
        ToolStatus::Error
    } else if warning_count > 0 {
        ToolStatus::Warn
    } else {
        ToolStatus::Ok
    };

    let summary = if error_count > 0 {
        format!("拒绝策略违规: {} 个错误", error_count)
    } else if warning_count > 0 {
        format!("拒绝策略: {} 个警告", warning_count)
    } else if exit_code != 0 {
        "检查失败".to_string()
    } else {
        "所有依赖符合策略".to_string()
    };

    let markdown_content = format!(
        "# Deny\n\n**命令**: `{command}`\n\n**状态**: {}\n\n**摘要**: {}\n\n## 输出\n\n```\n{}\n```\n",
        if exit_code == 0 && error_count == 0 {
            "✅ 通过"
        } else {
            "❌ 违规"
        },
        summary,
        combined.trim()
    );

    ToolReport {
        tool_name: "deny".to_string(),
        status,
        summary,
        output_path: "security/deny.md".to_string(),
        markdown_content,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::ToolStatus;

    #[test]
    fn test_deny_ok() {
        let r = parse(
            "advisories ok\nlicenses ok\nsources ok",
            "",
            0,
            "cargo deny check",
        );
        assert_eq!(r.status, ToolStatus::Ok);
    }

    #[test]
    fn test_deny_error() {
        let stderr = "error[denied]: crate 'openssl' is denied";
        let r = parse("", stderr, 1, "cargo deny check");
        assert_eq!(r.status, ToolStatus::Error);
    }
}
