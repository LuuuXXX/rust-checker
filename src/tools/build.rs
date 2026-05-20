use crate::report::{ToolReport, ToolStatus};

pub fn parse(stdout: &str, stderr: &str, exit_code: i32, command: &str) -> ToolReport {
    let combined = format!("{}\n{}", stdout, stderr);
    let status = if exit_code == 0 {
        ToolStatus::Ok
    } else {
        ToolStatus::Error
    };

    let error_count = combined
        .lines()
        .filter(|l| {
            (l.contains("error[") || l.starts_with("error:"))
                && !l.contains("could not compile")
                && !l.contains("aborting due to")
        })
        .count();
    let warning_count = combined
        .lines()
        .filter(|l| {
            l.contains("warning:")
                && !l.contains("warning emitted")
                && !l.contains("warnings emitted")
        })
        .count();

    let summary = if exit_code == 0 {
        if warning_count > 0 {
            format!("构建成功，{} 个警告", warning_count)
        } else {
            "构建成功".to_string()
        }
    } else {
        format!("构建失败，{} 个错误，{} 个警告", error_count, warning_count)
    };

    let markdown_content = format!(
        "# Build\n\n**命令**: `{command}`\n\n**状态**: {}\n\n**摘要**: {}\n\n## 输出\n\n```\n{}\n```\n",
        if exit_code == 0 { "✅ 成功" } else { "❌ 失败" },
        summary,
        combined.trim()
    );

    ToolReport {
        tool_name: "build".to_string(),
        status,
        summary,
        output_path: "quality/build.md".to_string(),
        markdown_content,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::ToolStatus;

    #[test]
    fn test_build_success() {
        let r = parse("", "Compiling foo v0.1.0\nFinished dev", 0, "cargo build");
        assert_eq!(r.status, ToolStatus::Ok);
        assert!(r.summary.contains("成功"));
    }

    #[test]
    fn test_build_failure() {
        let r = parse(
            "",
            "error[E0308]: mismatched types\nerror: aborting due to 1 error",
            1,
            "cargo build",
        );
        assert_eq!(r.status, ToolStatus::Error);
        assert!(r.summary.contains("失败"));
    }

    #[test]
    fn test_build_with_warnings() {
        let r = parse(
            "",
            "warning: unused variable `x`\nFinished dev",
            0,
            "cargo build",
        );
        assert_eq!(r.status, ToolStatus::Ok);
        assert!(r.summary.contains("警告"));
    }

    #[test]
    fn test_build_errors_excludes_could_not_compile_line() {
        // "error: could not compile `foo`" is a rustc summary line, not a diagnostic.
        let stderr = "error[E0308]: mismatched types\nerror: could not compile `foo` due to 1 previous error";
        let r = parse("", stderr, 1, "cargo build");
        assert_eq!(r.status, ToolStatus::Error);
        assert!(
            r.summary.contains("1"),
            "expected 1 error (not 2) in: {}",
            r.summary
        );
        assert!(
            !r.summary.contains("2"),
            "summary must not report 2 errors: {}",
            r.summary
        );
    }

    #[test]
    fn test_build_errors_excludes_aborting_due_to_line() {
        // "error: aborting due to N previous errors" is a rustc trailing summary, not a diagnostic.
        let stderr = "error[E0308]: mismatched types\nerror: aborting due to 1 previous error";
        let r = parse("", stderr, 1, "cargo build");
        assert_eq!(r.status, ToolStatus::Error);
        assert!(
            r.summary.contains("1"),
            "expected 1 error (not 2) in: {}",
            r.summary
        );
        assert!(
            !r.summary.contains("2"),
            "summary must not report 2 errors: {}",
            r.summary
        );
    }
}
