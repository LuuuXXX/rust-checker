use crate::report::{ToolReport, ToolStatus};

pub fn parse(stdout: &str, stderr: &str, exit_code: i32, command: &str) -> ToolReport {
    let combined = format!("{}\n{}", stdout, stderr);

    // Count dependency tree lines (lines with crate names)
    let dep_count = combined
        .lines()
        .filter(|l| {
            let trimmed = l.trim_start_matches(|c: char| {
                c == ' ' || c == '│' || c == '├' || c == '└' || c == '─'
            });
            !trimmed.is_empty() && !trimmed.starts_with('[')
        })
        .count();

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
    }

    #[test]
    fn test_deps_fail() {
        let r = parse("", "error", 1, "cargo tree");
        assert_eq!(r.status, ToolStatus::Error);
    }
}
