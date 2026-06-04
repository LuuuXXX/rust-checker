use crate::report::{ToolReport, ToolStatus};

pub fn parse(stdout: &str, stderr: &str, exit_code: i32, command: &str) -> ToolReport {
    let combined = format!("{}\n{}", stdout, stderr);
    let asan_error_count = combined.matches("ERROR: AddressSanitizer").count();

    let status = if exit_code != 0 || asan_error_count > 0 {
        ToolStatus::Error
    } else {
        ToolStatus::Ok
    };

    let summary = if asan_error_count > 0 {
        format!("ASan 发现 {} 个错误", asan_error_count)
    } else if exit_code != 0 {
        "ASan 构建失败".to_string()
    } else {
        "ASan 构建完成".to_string()
    };

    let error_section = extract_asan_error_section(&combined)
        .map(|section| format!("\n## ASan 错误摘要\n\n```\n{}\n```\n", section))
        .unwrap_or_default();

    let markdown_content = format!(
        "# ASan\n\n**命令**: `{command}`\n\n**状态**: {}\n\n**摘要**: {}{}\n\n## 输出\n\n```\n{}\n```\n",
        if status == ToolStatus::Ok {
            "✅ 通过"
        } else {
            "❌ 失败"
        },
        summary,
        error_section,
        combined.trim()
    );

    ToolReport {
        tool_name: "asan".to_string(),
        status,
        summary,
        output_path: "asan.md".to_string(),
        markdown_content,
    }
}

fn extract_asan_error_section(output: &str) -> Option<String> {
    let lines: Vec<&str> = output.lines().collect();
    let start = lines
        .iter()
        .position(|line| line.contains("ERROR: AddressSanitizer"))?;
    let end = lines[start..]
        .iter()
        .position(|line| line.contains("ABORTING"))
        .map(|offset| start + offset + 1)
        .unwrap_or_else(|| lines.len().min(start + 30));
    Some(lines[start..end].join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::ToolStatus;

    #[test]
    fn test_asan_build_ok() {
        let stderr = "Finished dev [unoptimized + debuginfo] target(s) in 1.0s";
        let r = parse("", stderr, 0, "cargo build");
        assert_eq!(r.status, ToolStatus::Ok);
        assert!(r.summary.contains("完成"));
    }

    #[test]
    fn test_asan_build_fail() {
        let r = parse(
            "",
            "error: the option `Z` is only accepted on nightly",
            1,
            "cargo build",
        );
        assert_eq!(r.status, ToolStatus::Error);
        assert!(r.summary.contains("失败"));
    }

    #[test]
    fn test_asan_runtime_error() {
        let stderr = "==1==ERROR: AddressSanitizer: heap-use-after-free\nstack\n==1==ABORTING";
        let r = parse("", stderr, 1, "cargo run");
        assert_eq!(r.status, ToolStatus::Error);
        assert!(r.summary.contains("1"));
        assert!(r.markdown_content.contains("heap-use-after-free"));
    }
}
