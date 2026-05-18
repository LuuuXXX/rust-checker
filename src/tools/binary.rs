use crate::report::{ToolReport, ToolStatus};

pub fn parse(stdout: &str, stderr: &str, exit_code: i32, command: &str) -> ToolReport {
    let combined = format!("{}\n{}", stdout, stderr);

    // Extract binary info from cargo build output
    // Look for lines mentioning binary size or output path
    let binary_lines: Vec<&str> = combined
        .lines()
        .filter(|l| {
            l.contains("Compiling") || l.contains("Finished") || l.contains("target/")
                || l.contains(".exe")
        })
        .collect();

    // Try to get binary path
    let binary_path = combined
        .lines()
        .find(|l| l.contains("target/") && (l.contains("/release/") || l.contains("/debug/")))
        .map(|l| l.trim().to_string());

    let status = if exit_code != 0 {
        ToolStatus::Error
    } else {
        ToolStatus::Ok
    };

    let summary = if exit_code != 0 {
        "二进制信息获取失败".to_string()
    } else if let Some(path) = &binary_path {
        format!("二进制路径: {}", path)
    } else if !binary_lines.is_empty() {
        format!("构建信息: {} 行", binary_lines.len())
    } else {
        "二进制信息已收集".to_string()
    };

    let markdown_content = format!(
        "# Binary\n\n**命令**: `{command}`\n\n**状态**: {}\n\n**摘要**: {}\n\n## 输出\n\n```\n{}\n```\n",
        if exit_code == 0 { "✅ 成功" } else { "❌ 失败" },
        summary,
        combined.trim()
    );

    ToolReport {
        tool_name: "binary".to_string(),
        status,
        summary,
        output_path: "compat/binary.md".to_string(),
        markdown_content,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::ToolStatus;

    #[test]
    fn test_binary_ok() {
        let stderr = "Compiling foo v0.1.0\nFinished release [optimized] target(s) in 5.2s\ntarget/release/foo";
        let r = parse("", stderr, 0, "cargo build --release");
        assert_eq!(r.status, ToolStatus::Ok);
    }

    #[test]
    fn test_binary_fail() {
        let r = parse("", "error: linker error", 1, "cargo build --release");
        assert_eq!(r.status, ToolStatus::Error);
    }
}
