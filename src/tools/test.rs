use crate::report::{ToolReport, ToolStatus};

pub fn parse(stdout: &str, stderr: &str, exit_code: i32, command: &str) -> ToolReport {
    let combined = format!("{}\n{}", stdout, stderr);

    // Look for "test result: ok. N passed; M failed;"
    let mut passed = 0usize;
    let mut failed = 0usize;
    let mut ignored = 0usize;
    let mut found_result = false;

    for line in combined.lines() {
        if line.contains("test result:") {
            found_result = true;
            // "test result: ok. 5 passed; 0 failed; 0 ignored"
            // Accumulate across all test binaries (unit tests, integration tests, etc.)
            if let Some(p) = extract_number(line, "passed") {
                passed += p;
            }
            if let Some(f) = extract_number(line, "failed") {
                failed += f;
            }
            if let Some(i) = extract_number(line, "ignored") {
                ignored += i;
            }
        }
    }

    let status = if exit_code == 0 {
        ToolStatus::Ok
    } else {
        ToolStatus::Error
    };

    let summary = if found_result {
        format!("通过: {}，失败: {}，忽略: {}", passed, failed, ignored)
    } else if exit_code == 0 {
        "测试通过".to_string()
    } else {
        "测试失败".to_string()
    };

    let markdown_content = format!(
        "# Test\n\n**命令**: `{command}`\n\n**状态**: {}\n\n**摘要**: {}\n\n## 输出\n\n```\n{}\n```\n",
        if exit_code == 0 { "✅ 成功" } else { "❌ 失败" },
        summary,
        combined.trim()
    );

    ToolReport {
        tool_name: "test".to_string(),
        status,
        summary,
        output_path: "quality/test.md".to_string(),
        markdown_content,
    }
}

fn extract_number(line: &str, keyword: &str) -> Option<usize> {
    let idx = line.find(keyword)?;
    let before = line[..idx].trim();
    let num_str = before.split_whitespace().last()?;
    num_str.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::ToolStatus;

    #[test]
    fn test_parse_success() {
        let stdout =
            "running 3 tests\ntest foo ... ok\ntest result: ok. 3 passed; 0 failed; 0 ignored";
        let r = parse(stdout, "", 0, "cargo test");
        assert_eq!(r.status, ToolStatus::Ok);
        assert!(r.summary.contains("3"));
    }

    #[test]
    fn test_parse_failure() {
        let stdout = "test result: FAILED. 2 passed; 1 failed; 0 ignored";
        let r = parse(stdout, "", 1, "cargo test");
        assert_eq!(r.status, ToolStatus::Error);
    }

    #[test]
    fn test_parse_empty() {
        let r = parse("", "", 0, "cargo test");
        assert_eq!(r.status, ToolStatus::Ok);
    }

    #[test]
    fn test_parse_multiple_result_lines_are_accumulated() {
        // cargo test prints one "test result:" line per test binary;
        // counts must be summed across all binaries.
        let stdout = concat!(
            "test result: ok. 133 passed; 0 failed; 2 ignored; finished in 0.05s\n",
            "test result: ok. 0 passed; 0 failed; 0 ignored; finished in 0.00s\n",
            "test result: ok. 18 passed; 1 failed; 0 ignored; finished in 0.03s\n",
        );
        let r = parse(stdout, "", 1, "cargo test");
        assert_eq!(r.status, ToolStatus::Error);
        // 133+0+18 = 151, 0+0+1 = 1, 2+0+0 = 2
        assert!(
            r.summary.contains("151"),
            "expected 151 passed in: {}",
            r.summary
        );
        assert!(
            r.summary.contains("失败: 1"),
            "expected '失败: 1' in: {}",
            r.summary
        );
    }
}
