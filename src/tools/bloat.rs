use crate::report::{ToolReport, ToolStatus};

pub fn parse(stdout: &str, stderr: &str, exit_code: i32, command: &str) -> ToolReport {
    let combined = format!("{}\n{}", stdout, stderr);

    // cargo bloat outputs a table of functions/sections with sizes
    // Look for the total binary size line
    let mut total_size: Option<String> = None;
    let mut function_count = 0usize;

    for line in combined.lines() {
        let lower = line.to_lowercase();
        if lower.contains("file")
            && (lower.contains("kb") || lower.contains("mb") || lower.contains("bytes"))
        {
            // Extract size info
            for part in line.split_whitespace() {
                if part.ends_with("KB") || part.ends_with("MB") || part.ends_with('B') {
                    total_size = Some(part.to_string());
                    break;
                }
            }
        }
        // Count function/section rows (lines with %)
        if line.contains('%') && !lower.contains("file") && !lower.contains("total") {
            function_count += 1;
        }
    }

    let status = if exit_code != 0 {
        ToolStatus::Error
    } else {
        ToolStatus::Ok
    };

    let summary = if exit_code != 0 {
        "Bloat 分析失败".to_string()
    } else if let Some(size) = &total_size {
        format!("二进制大小: {}，分析了 {} 个函数", size, function_count)
    } else {
        format!("Bloat 分析完成，分析了 {} 项", function_count)
    };

    let markdown_content = format!(
        "# Bloat\n\n**命令**: `{command}`\n\n**状态**: {}\n\n**摘要**: {}\n\n## 输出\n\n```\n{}\n```\n",
        if exit_code == 0 { "✅ 成功" } else { "❌ 失败" },
        summary,
        combined.trim()
    );

    ToolReport {
        tool_name: "bloat".to_string(),
        status,
        summary,
        output_path: "perf/bloat.md".to_string(),
        markdown_content,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::ToolStatus;

    #[test]
    fn test_bloat_ok() {
        let stdout = " File  .text   Size          Crate Name\n  2.3%  2.5%  1.0KB          foo bar_fn\n File size: 45KB";
        let r = parse(stdout, "", 0, "cargo bloat");
        assert_eq!(r.status, ToolStatus::Ok);
    }

    #[test]
    fn test_bloat_fail() {
        let r = parse("", "error: failed", 1, "cargo bloat");
        assert_eq!(r.status, ToolStatus::Error);
    }
}
