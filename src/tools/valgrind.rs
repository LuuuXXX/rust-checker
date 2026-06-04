use crate::report::{ToolReport, ToolStatus};

pub fn parse(
    tool_name: &str,
    stdout: &str,
    stderr: &str,
    exit_code: i32,
    command: &str,
) -> ToolReport {
    let combined = format!("{}\n{}", stdout, stderr);
    let error_count = parse_error_summary(&combined);
    let leak_lines: Vec<&str> = combined
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            (trimmed.contains("definitely lost:")
                || trimmed.contains("indirectly lost:")
                || trimmed.contains("possibly lost:"))
                && !trimmed.contains(": 0 bytes")
        })
        .collect();

    let status = if exit_code != 0 || error_count.unwrap_or(0) > 0 {
        ToolStatus::Error
    } else if !leak_lines.is_empty() {
        ToolStatus::Warn
    } else {
        ToolStatus::Ok
    };

    let summary = if exit_code != 0 {
        match error_count {
            Some(errors) => format!("Valgrind check failed, {errors} errors"),
            None => "Valgrind check failed".to_string(),
        }
    } else if let Some(errors) = error_count {
        if errors > 0 {
            format!("Valgrind found {errors} errors")
        } else if !leak_lines.is_empty() {
            format!("Valgrind found {} leak hints", leak_lines.len())
        } else {
            "Valgrind found no errors".to_string()
        }
    } else if !leak_lines.is_empty() {
        format!("Valgrind found {} leak hints", leak_lines.len())
    } else {
        "Valgrind check completed".to_string()
    };

    let leak_section = if leak_lines.is_empty() {
        String::new()
    } else {
        format!("\n## Leak Summary\n\n```\n{}\n```\n", leak_lines.join("\n"))
    };

    let markdown_content = format!(
        "# {}\n\n**Command**: `{command}`\n\n**Status**: {}\n\n**Summary**: {}{}\n\n## Output\n\n```\n{}\n```\n",
        display_name(tool_name),
        match status {
            ToolStatus::Ok => "ok",
            ToolStatus::Warn => "warning",
            ToolStatus::Error => "failed",
            ToolStatus::Skipped => "skipped",
        },
        summary,
        leak_section,
        combined.trim()
    );

    ToolReport {
        tool_name: tool_name.to_string(),
        status,
        summary,
        output_path: crate::config::effective_output_path(tool_name, None),
        markdown_content,
    }
}

fn display_name(tool_name: &str) -> &'static str {
    match tool_name {
        "valgrind_memcheck" => "Valgrind Memcheck",
        "valgrind_helgrind" => "Valgrind Helgrind",
        "valgrind_drd" => "Valgrind DRD",
        _ => "Valgrind",
    }
}

fn parse_error_summary(output: &str) -> Option<usize> {
    output.lines().find_map(|line| {
        let (_, rest) = line.split_once("ERROR SUMMARY:")?;
        rest.split_whitespace().next()?.parse().ok()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::ToolStatus;

    #[test]
    fn test_valgrind_ok_with_zero_error_summary() {
        let stderr = "==123== ERROR SUMMARY: 0 errors from 0 contexts";
        let r = parse("valgrind_memcheck", "", stderr, 0, "cargo valgrind");
        assert_eq!(r.status, ToolStatus::Ok);
        assert!(r.summary.contains("no errors"));
    }

    #[test]
    fn test_valgrind_error_summary_is_error() {
        let stderr = "==123== ERROR SUMMARY: 2 errors from 1 contexts";
        let r = parse("valgrind_memcheck", "", stderr, 0, "cargo valgrind");
        assert_eq!(r.status, ToolStatus::Error);
        assert!(r.summary.contains("2"));
    }

    #[test]
    fn test_valgrind_nonzero_exit_is_error() {
        let r = parse(
            "valgrind_memcheck",
            "",
            "valgrind failed",
            1,
            "cargo valgrind",
        );
        assert_eq!(r.status, ToolStatus::Error);
    }

    #[test]
    fn test_valgrind_leak_without_error_summary_is_warn() {
        let stderr = "==123== definitely lost: 24 bytes in 1 blocks";
        let r = parse("valgrind_memcheck", "", stderr, 0, "cargo valgrind");
        assert_eq!(r.status, ToolStatus::Warn);
        assert!(r.summary.contains("leak"));
        assert!(r.markdown_content.contains("## Leak Summary"));
    }

    #[test]
    fn test_valgrind_split_tool_output_path() {
        let r = parse(
            "valgrind_helgrind",
            "",
            "==123== ERROR SUMMARY: 0 errors from 0 contexts",
            0,
            "cargo valgrind",
        );
        assert_eq!(r.tool_name, "valgrind_helgrind");
        assert_eq!(r.output_path, "valgrind/helgrind.md");
        assert!(r.markdown_content.contains("# Valgrind Helgrind"));
    }
}
