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
            if let Some(p) = extract_number(line, "passed") {
                passed = p;
            }
            if let Some(f) = extract_number(line, "failed") {
                failed = f;
            }
            if let Some(i) = extract_number(line, "ignored") {
                ignored = i;
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
        let stdout = "running 3 tests\ntest foo ... ok\ntest result: ok. 3 passed; 0 failed; 0 ignored";
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
}
