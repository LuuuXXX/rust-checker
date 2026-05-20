use crate::report::{ToolReport, ToolStatus};

pub fn parse(stdout: &str, stderr: &str, exit_code: i32, command: &str) -> ToolReport {
    let combined = format!("{}\n{}", stdout, stderr);

    // Look for unsafe counts in Total line
    // cargo geiger outputs something like: "Total   | 0 | 0 | 0 | 0 | 0 |"
    let mut unsafe_count = 0usize;
    for line in combined.lines() {
        let lower = line.to_lowercase();
        if lower
            .split_whitespace()
            .next()
            .is_some_and(|w| w == "total")
            && line.contains('|')
        {
            // Parse numbers from the total line
            let numbers: Vec<usize> = line
                .split('|')
                .filter_map(|s| s.trim().parse::<usize>().ok())
                .collect();
            unsafe_count = numbers.iter().sum();
            break;
        }
        // Also look for "unsafe functions:" pattern (older geiger versions)
        if lower.contains("unsafe functions:") {
            for part in line.split_whitespace() {
                if let Ok(n) = part.parse::<usize>() {
                    unsafe_count = unsafe_count.saturating_add(n);
                }
            }
        }
    }

    let status = if exit_code != 0 {
        ToolStatus::Error
    } else if unsafe_count > 0 {
        ToolStatus::Warn
    } else {
        ToolStatus::Ok
    };

    let summary = if exit_code != 0 {
        "Geiger 检查失败".to_string()
    } else if unsafe_count > 0 {
        format!("发现 {} 处不安全代码", unsafe_count)
    } else {
        "未发现不安全代码".to_string()
    };

    let markdown_content = format!(
        "# Geiger\n\n**命令**: `{command}`\n\n**状态**: {}\n\n**摘要**: {}\n\n## 输出\n\n```\n{}\n```\n",
        match status {
            ToolStatus::Ok => "✅ 安全",
            ToolStatus::Warn => "⚠️ 存在 unsafe",
            _ => "❌ 失败",
        },
        summary,
        combined.trim()
    );

    ToolReport {
        tool_name: "geiger".to_string(),
        status,
        summary,
        output_path: "security/geiger.md".to_string(),
        markdown_content,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::ToolStatus;

    #[test]
    fn test_geiger_safe() {
        let r = parse("No unsafe code found", "", 0, "cargo geiger");
        assert_eq!(r.status, ToolStatus::Ok);
    }

    #[test]
    fn test_geiger_unsafe() {
        let stdout = "Functions  |  0  |  5  |  0  |  0 |\nTotal      |  0  |  5  |  0  |  0 |";
        let r = parse(stdout, "", 0, "cargo geiger");
        assert_eq!(r.status, ToolStatus::Warn);
    }

    #[test]
    fn test_geiger_failure() {
        let r = parse("", "error: failed", 1, "cargo geiger");
        assert_eq!(r.status, ToolStatus::Error);
    }

    #[test]
    fn test_geiger_total_line_not_confused_by_subtotal() {
        // A crate whose name starts with "total" must not match the total line
        // (first-word check instead of substring).  The real Total row comes last.
        let stdout =
            "total-crate v1.0  |  0  |  3  |  0  |  0 |\nTotal         |  0  |  1  |  0  |  0 |";
        let r = parse(stdout, "", 0, "cargo geiger");
        assert_eq!(r.status, ToolStatus::Warn);
        assert!(
            r.summary.contains("1"),
            "should use Total row (1), not total-crate row (3): {}",
            r.summary
        );
    }
}
