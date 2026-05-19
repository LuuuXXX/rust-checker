use crate::report::{ToolReport, ToolStatus};

pub fn parse(stdout: &str, stderr: &str, exit_code: i32, command: &str) -> ToolReport {
    let combined = format!("{}\n{}", stdout, stderr);

    // cargo fmt --check exits non-zero if files need formatting
    let status = if exit_code == 0 {
        ToolStatus::Ok
    } else {
        ToolStatus::Warn
    };

    // Count distinct files that would be reformatted (each file produces one "Diff in" line)
    let diff_count = combined.lines().filter(|l| l.contains("Diff in")).count();

    let summary = if exit_code == 0 {
        "代码格式正确".to_string()
    } else if diff_count > 0 {
        format!("需要格式化: {} 个文件", diff_count)
    } else {
        "存在格式问题".to_string()
    };

    let markdown_content = format!(
        "# Fmt\n\n**命令**: `{command}`\n\n**状态**: {}\n\n**摘要**: {}\n\n## 输出\n\n```\n{}\n```\n",
        if exit_code == 0 { "✅ 格式正确" } else { "⚠️ 需要格式化" },
        summary,
        combined.trim()
    );

    ToolReport {
        tool_name: "fmt".to_string(),
        status,
        summary,
        output_path: "quality/fmt.md".to_string(),
        markdown_content,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::ToolStatus;

    #[test]
    fn test_fmt_ok() {
        let r = parse("", "", 0, "cargo fmt --check");
        assert_eq!(r.status, ToolStatus::Ok);
        assert!(r.summary.contains("正确"));
    }

    #[test]
    fn test_fmt_needs_formatting() {
        // One file produces one "Diff in" line plus --- and +++ headers
        let stderr = "Diff in src/main.rs at line 5:\n--- src/main.rs\n+++ src/main.rs";
        let r = parse("", stderr, 1, "cargo fmt --check");
        assert_eq!(r.status, ToolStatus::Warn);
        // Should report exactly 1 file, not 3 (not counting --- / +++ headers)
        assert!(r.summary.contains("1 个文件"), "got: {}", r.summary);
    }

    #[test]
    fn test_fmt_multiple_files() {
        let stderr = "Diff in src/main.rs at line 5:\n--- src/main.rs\n+++ src/main.rs\nDiff in src/lib.rs at line 2:\n--- src/lib.rs\n+++ src/lib.rs";
        let r = parse("", stderr, 1, "cargo fmt --check");
        assert_eq!(r.status, ToolStatus::Warn);
        assert!(r.summary.contains("2 个文件"), "got: {}", r.summary);
    }
}
