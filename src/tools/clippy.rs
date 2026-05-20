use crate::report::{ToolReport, ToolStatus};

pub fn parse(stdout: &str, stderr: &str, exit_code: i32, command: &str) -> ToolReport {
    let combined = format!("{}\n{}", stdout, stderr);

    let warning_count = combined
        .lines()
        .filter(|l| {
            l.contains("warning:")
                && !l.contains("warning emitted")
                && !l.contains("warnings emitted")
        })
        .count();
    let error_count = combined
        .lines()
        .filter(|l| {
            l.contains("error:")
                && !l.contains("aborting due to")
                && !l.contains("could not compile")
        })
        .count();

    let status = if error_count > 0 || exit_code != 0 {
        ToolStatus::Error
    } else if warning_count > 0 {
        ToolStatus::Warn
    } else {
        ToolStatus::Ok
    };

    let summary = if error_count > 0 {
        format!("{} 个错误，{} 个警告", error_count, warning_count)
    } else if warning_count > 0 {
        format!("{} 个警告", warning_count)
    } else {
        "无问题".to_string()
    };

    let markdown_content = format!(
        "# Clippy\n\n**命令**: `{command}`\n\n**状态**: {}\n\n**摘要**: {}\n\n## 输出\n\n```\n{}\n```\n",
        match status {
            ToolStatus::Ok => "✅ 通过",
            ToolStatus::Warn => "⚠️ 警告",
            _ => "❌ 失败",
        },
        summary,
        combined.trim()
    );

    ToolReport {
        tool_name: "clippy".to_string(),
        status,
        summary,
        output_path: "quality/clippy.md".to_string(),
        markdown_content,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::ToolStatus;

    #[test]
    fn test_clippy_clean() {
        let r = parse("", "", 0, "cargo clippy");
        assert_eq!(r.status, ToolStatus::Ok);
        assert_eq!(r.summary, "无问题");
    }

    #[test]
    fn test_clippy_warnings() {
        let stderr = "warning: unused variable `x`\nwarning: 1 warning emitted";
        let r = parse("", stderr, 0, "cargo clippy");
        assert_eq!(r.status, ToolStatus::Warn);
        assert!(r.summary.contains("警告"));
    }

    #[test]
    fn test_clippy_warnings_excludes_emitted_summary_line() {
        // "N warnings emitted" is a rustc/clippy trailing summary line and must not
        // be counted as an additional warning itself.
        let stderr =
            "warning: unused variable `x`\nwarning: dead_code\nwarning: 2 warnings emitted";
        let r = parse("", stderr, 0, "cargo clippy");
        assert_eq!(r.status, ToolStatus::Warn);
        assert!(
            r.summary.contains("2"),
            "expected 2 warnings (not 3) in: {}",
            r.summary
        );
        assert!(
            !r.summary.contains("3"),
            "summary must not report 3 warnings: {}",
            r.summary
        );
    }

    #[test]
    fn test_clippy_errors() {
        let stderr = "error: expected identifier\nerror: aborting due to 1 previous error";
        let r = parse("", stderr, 1, "cargo clippy");
        assert_eq!(r.status, ToolStatus::Error);
    }

    #[test]
    fn test_clippy_errors_excludes_could_not_compile_line() {
        // "error: could not compile `foo`" is a rustc trailing summary line, not a diagnostic.
        let stderr = "error: unused import\nerror: could not compile `foo` due to 1 previous error";
        let r = parse("", stderr, 1, "cargo clippy");
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
