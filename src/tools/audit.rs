use crate::report::{ToolReport, ToolStatus};

pub fn parse(stdout: &str, stderr: &str, exit_code: i32, command: &str) -> ToolReport {
    let combined = format!("{}\n{}", stdout, stderr);

    // cargo audit outputs JSON or text. Look for vulnerability indicators.
    let vuln_count = count_vulnerabilities(&combined);

    let status = if exit_code != 0 || vuln_count > 0 {
        ToolStatus::Error
    } else {
        ToolStatus::Ok
    };

    let summary = if vuln_count > 0 {
        format!("发现 {} 个漏洞", vuln_count)
    } else if exit_code != 0 {
        "安全审计失败".to_string()
    } else {
        "未发现已知漏洞".to_string()
    };

    let markdown_content = format!(
        "# Audit\n\n**命令**: `{command}`\n\n**状态**: {}\n\n**摘要**: {}\n\n## 输出\n\n```\n{}\n```\n",
        if vuln_count == 0 && exit_code == 0 {
            "✅ 安全"
        } else {
            "❌ 存在漏洞"
        },
        summary,
        combined.trim()
    );

    ToolReport {
        tool_name: "audit".to_string(),
        status,
        summary,
        output_path: "security/audit.md".to_string(),
        markdown_content,
    }
}

fn count_vulnerabilities(output: &str) -> usize {
    // Try to find "Vulnerabilities found: N"
    for line in output.lines() {
        let lower = line.to_lowercase();
        if lower.contains("vulnerabilities found") || lower.contains("vulnerability found") {
            for part in line.split_whitespace() {
                if let Ok(n) = part.parse::<usize>() {
                    return n;
                }
            }
        }
        // "error[" lines indicate vulnerabilities in text mode
        if line.contains("error[") && line.contains("RUSTSEC") {
            return 1; // at least one
        }
    }

    // Check for JSON vulnerability count
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(output) {
        if let Some(count) = json
            .get("vulnerabilities")
            .and_then(|v| v.get("count"))
            .and_then(|c| c.as_u64())
        {
            return count as usize;
        }
    }

    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::ToolStatus;

    #[test]
    fn test_audit_clean() {
        let r = parse("", "0 vulnerabilities found", 0, "cargo audit");
        assert_eq!(r.status, ToolStatus::Ok);
    }

    #[test]
    fn test_audit_vuln() {
        let r = parse(
            "",
            "error[RUSTSEC-2022-0001]: vuln\n1 vulnerabilities found",
            1,
            "cargo audit",
        );
        assert_eq!(r.status, ToolStatus::Error);
    }

    #[test]
    fn test_audit_empty() {
        let r = parse("", "", 0, "cargo audit");
        assert_eq!(r.status, ToolStatus::Ok);
    }

    #[test]
    fn test_audit_json_vuln_count() {
        // cargo audit --json produces a JSON object; the vulnerabilities.count field
        // should be picked up by the JSON parsing branch.
        let json = r#"{"vulnerabilities":{"count":2,"list":[]},"warnings":{}}"#;
        let r = parse(json, "", 1, "cargo audit --json");
        assert_eq!(r.status, ToolStatus::Error);
        assert!(
            r.summary.contains("2"),
            "expected vuln count in: {}",
            r.summary
        );
    }

    #[test]
    fn test_audit_json_no_vulns() {
        let json = r#"{"vulnerabilities":{"count":0,"list":[]},"warnings":{}}"#;
        let r = parse(json, "", 0, "cargo audit --json");
        assert_eq!(r.status, ToolStatus::Ok);
    }
}
