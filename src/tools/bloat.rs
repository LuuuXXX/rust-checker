use crate::report::{ToolReport, ToolStatus};

pub fn parse(stdout: &str, stderr: &str, exit_code: i32, command: &str) -> ToolReport {
    let combined = format!("{}\n{}", stdout, stderr);

    // cargo bloat outputs a table of functions/sections with sizes
    // Look for the total binary size line
    let mut total_size: Option<String> = None;
    let mut function_count = 0usize;

    for line in combined.lines() {
        let lower = line.to_lowercase();
        // real cargo bloat uses KiB/MiB (kibibyte units); also accept plain KB/MB/B
        if lower.contains("file")
            && (lower.contains("kb")
                || lower.contains("kib")
                || lower.contains("mb")
                || lower.contains("mib")
                || lower.contains("bytes"))
        {
            // Extract size info — real cargo bloat uses KiB/MiB; also accept plain KB/MB/B/bytes
            for part in line.split_whitespace() {
                if part.ends_with("KiB")
                    || part.ends_with("MiB")
                    || part.ends_with("KB")
                    || part.ends_with("MB")
                    || part.ends_with('B')
                    || part.to_lowercase() == "bytes"
                {
                    // For a bare "bytes" token, grab the preceding numeric token as the size.
                    if part.to_lowercase() == "bytes" {
                        // We re-scan the line to pick up "N bytes" where N is the previous token.
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if let Some(pos) = parts.iter().position(|p| p.to_lowercase() == "bytes") {
                            if pos > 0 && parts[pos - 1].parse::<u64>().is_ok() {
                                total_size = Some(format!("{} bytes", parts[pos - 1]));
                            }
                        }
                    } else {
                        total_size = Some(part.to_string());
                    }
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
    fn test_bloat_kib_format() {
        // Real cargo bloat uses KiB/MiB units (kibibytes), not plain KB/MB.
        let stdout = " 10.2%  11.0%  1.2KiB          my_crate foo_fn\n File .text size: 512KiB";
        let r = parse(stdout, "", 0, "cargo bloat");
        assert_eq!(r.status, ToolStatus::Ok);
        assert!(
            r.summary.contains("KiB"),
            "KiB size must appear in summary: {}",
            r.summary
        );
    }

    #[test]
    fn test_bloat_fail() {
        let r = parse("", "error: failed", 1, "cargo bloat");
        assert_eq!(r.status, ToolStatus::Error);
    }

    #[test]
    fn test_bloat_bytes_format() {
        // "N bytes" (space-separated) must also be recognised.
        let stdout = " File  .text   Size          Crate Name\n  1.0%  1.2%   128 bytes          foo f\n File size: 1024 bytes";
        let r = parse(stdout, "", 0, "cargo bloat");
        assert_eq!(r.status, ToolStatus::Ok);
        assert!(
            r.summary.contains("bytes"),
            "space-separated bytes size must appear in summary: {}",
            r.summary
        );
    }
}
