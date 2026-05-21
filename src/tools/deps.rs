use crate::report::{ToolReport, ToolStatus};

pub fn parse(stdout: &str, stderr: &str, exit_code: i32, command: &str) -> ToolReport {
    let combined = format!("{}\n{}", stdout, stderr);

    // Count dependency tree lines (lines with crate names), excluding the first
    // line which is the root crate itself (no leading tree-drawing characters).
    let mut dep_count = 0usize;
    let mut root_seen = false;
    for line in combined.lines() {
        let trimmed = line.trim_start_matches([' ', '│', '├', '└', '─']);
        if trimmed.is_empty() || trimmed.starts_with('[') {
            continue;
        }
        if !root_seen {
            // First non-empty, non-bracket line is the root crate — skip it.
            root_seen = true;
            continue;
        }
        dep_count += 1;
    }

    let status = if exit_code != 0 {
        ToolStatus::Error
    } else {
        ToolStatus::Ok
    };

    let summary = if exit_code != 0 {
        "依赖树生成失败".to_string()
    } else {
        format!("依赖树共 {} 项", dep_count)
    };

    let markdown_content = format!(
        "# Deps\n\n**命令**: `{command}`\n\n**状态**: {}\n\n**摘要**: {}\n\n## 输出\n\n```\n{}\n```\n",
        if exit_code == 0 { "✅ 成功" } else { "❌ 失败" },
        summary,
        combined.trim()
    );

    ToolReport {
        tool_name: "deps".to_string(),
        status,
        summary,
        output_path: "deps/deps.md".to_string(),
        markdown_content,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::ToolStatus;

    #[test]
    fn test_deps_ok() {
        let stdout = "my-crate v0.1.0\n├── serde v1.0\n│   └── serde_derive v1.0\n└── tokio v1.0";
        let r = parse(stdout, "", 0, "cargo tree");
        assert_eq!(r.status, ToolStatus::Ok);
        // 3 dependencies, root crate excluded
        assert!(
            r.summary.contains("3"),
            "expected 3 deps (root excluded), got: {}",
            r.summary
        );
    }

    #[test]
    fn test_deps_fail() {
        let r = parse("", "error", 1, "cargo tree");
        assert_eq!(r.status, ToolStatus::Error);
    }
}
