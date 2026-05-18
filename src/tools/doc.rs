use crate::report::{ToolReport, ToolStatus};

pub fn parse(stdout: &str, stderr: &str, exit_code: i32, command: &str) -> ToolReport {
    let combined = format!("{}\n{}", stdout, stderr);

    let warning_count = combined.lines().filter(|l| l.contains("warning:")).count();

    let status = if exit_code != 0 {
        ToolStatus::Error
    } else if warning_count > 0 {
        ToolStatus::Warn
    } else {
        ToolStatus::Ok
    };

    let summary = if exit_code != 0 {
        "文档生成失败".to_string()
    } else if warning_count > 0 {
        format!("文档生成成功，{} 个警告", warning_count)
    } else {
        "文档生成成功".to_string()
    };

    let markdown_content = format!(
        "# Doc\n\n**命令**: `{command}`\n\n**状态**: {}\n\n**摘要**: {}\n\n## 输出\n\n```\n{}\n```\n",
        if exit_code == 0 { "✅ 成功" } else { "❌ 失败" },
        summary,
        combined.trim()
    );

    ToolReport {
        tool_name: "doc".to_string(),
        status,
        summary,
        output_path: "quality/doc.md".to_string(),
        markdown_content,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::ToolStatus;

    #[test]
    fn test_doc_ok() {
        let r = parse("", "Documenting foo v0.1.0\nFinished", 0, "cargo doc");
        assert_eq!(r.status, ToolStatus::Ok);
    }

    #[test]
    fn test_doc_warning() {
        let r = parse(
            "",
            "warning: missing documentation for struct `Foo`",
            0,
            "cargo doc",
        );
        assert_eq!(r.status, ToolStatus::Warn);
        assert!(r.summary.contains("警告"));
    }
}
