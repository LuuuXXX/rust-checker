use crate::report::{ToolReport, ToolStatus};

pub fn parse(stdout: &str, stderr: &str, exit_code: i32, command: &str) -> ToolReport {
    let combined = format!("{}\n{}", stdout, stderr);

    // Look for unused dependencies section
    let has_unused = combined
        .lines()
        .any(|l| l.contains("unused dependencies") || l.contains("unused crates"));

    // Real cargo-udeps lists unused deps as bullet lines under the header:
    //   * "serde" (normal)
    //   * "tokio" (dev-build)
    // Count those bullet lines.  If none are found, `has_unused` still sets Warn status.
    let unused_count = combined
        .lines()
        .filter(|l| {
            let t = l.trim();
            t.starts_with("* \"") || t.starts_with("* '")
        })
        .count();

    let status = if exit_code != 0 {
        ToolStatus::Error
    } else if has_unused || unused_count > 0 {
        ToolStatus::Warn
    } else {
        ToolStatus::Ok
    };

    let summary = if exit_code != 0 {
        "未使用依赖检查失败".to_string()
    } else if unused_count > 0 {
        format!("发现 {} 个未使用依赖", unused_count)
    } else if has_unused {
        "存在未使用依赖".to_string()
    } else {
        "无未使用依赖".to_string()
    };

    let markdown_content = format!(
        "# Udeps\n\n**命令**: `{command}`\n\n**状态**: {}\n\n**摘要**: {}\n\n## 输出\n\n```\n{}\n```\n",
        match status {
            ToolStatus::Ok => "✅ 干净",
            ToolStatus::Warn => "⚠️ 存在未用依赖",
            _ => "❌ 失败",
        },
        summary,
        combined.trim()
    );

    ToolReport {
        tool_name: "udeps".to_string(),
        status,
        summary,
        output_path: "deps/udeps.md".to_string(),
        markdown_content,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::ToolStatus;

    #[test]
    fn test_udeps_clean() {
        let r = parse("All dependencies are used.", "", 0, "cargo udeps");
        assert_eq!(r.status, ToolStatus::Ok);
    }

    #[test]
    fn test_udeps_unused() {
        // Real cargo-udeps bullet format under a header line.
        let stdout =
            "warning: unused dependencies:\n * \"serde\" (normal)\n * \"tokio\" (dev-build)";
        let r = parse(stdout, "", 0, "cargo udeps");
        assert_eq!(r.status, ToolStatus::Warn);
        assert!(
            r.summary.contains("2"),
            "must count bullet-format entries: {}",
            r.summary
        );
    }

    #[test]
    fn test_udeps_unused_single_quote_variant() {
        // Defensive: also accept single-quote bullet variant.
        let stdout = "warning: unused dependencies:\n * 'serde' (normal)";
        let r = parse(stdout, "", 0, "cargo udeps");
        assert_eq!(r.status, ToolStatus::Warn);
        assert!(
            r.summary.contains("1"),
            "single-quote bullet must be counted: {}",
            r.summary
        );
    }

    #[test]
    fn test_udeps_fail() {
        let r = parse("", "error", 1, "cargo udeps");
        assert_eq!(r.status, ToolStatus::Error);
    }
}
