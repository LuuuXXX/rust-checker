use crate::report::{ToolReport, ToolStatus};

pub fn parse(stdout: &str, stderr: &str, exit_code: i32, command: &str) -> ToolReport {
    let combined = format!("{}\n{}", stdout, stderr);

    // Check for SVG file generation
    let svg_generated = combined.lines().any(|l| l.contains(".svg") || l.contains("flamegraph"));

    let status = if exit_code != 0 {
        ToolStatus::Error
    } else {
        ToolStatus::Ok
    };

    let summary = if exit_code != 0 {
        "Flamegraph 生成失败".to_string()
    } else if svg_generated {
        "Flamegraph SVG 已生成".to_string()
    } else {
        "Flamegraph 运行完成".to_string()
    };

    // Extract SVG path if present
    let svg_path = combined
        .lines()
        .find(|l| l.contains(".svg"))
        .map(|l| {
            l.split_whitespace()
                .find(|p| p.ends_with(".svg"))
                .unwrap_or(l)
                .to_string()
        });

    let svg_section = if let Some(path) = svg_path {
        format!("\n## 火焰图\n\n查看生成的 SVG 文件: `{}`\n", path)
    } else {
        String::new()
    };

    let markdown_content = format!(
        "# Flamegraph\n\n**命令**: `{command}`\n\n**状态**: {}\n\n**摘要**: {}{}\n\n## 输出\n\n```\n{}\n```\n",
        if exit_code == 0 { "✅ 成功" } else { "❌ 失败" },
        summary,
        svg_section,
        combined.trim()
    );

    ToolReport {
        tool_name: "flamegraph".to_string(),
        status,
        summary,
        output_path: "perf/flamegraph.md".to_string(),
        markdown_content,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::ToolStatus;

    #[test]
    fn test_flamegraph_ok() {
        let stdout = "Writing flamegraph.svg";
        let r = parse(stdout, "", 0, "cargo flamegraph");
        assert_eq!(r.status, ToolStatus::Ok);
        assert!(r.summary.contains("SVG"));
    }

    #[test]
    fn test_flamegraph_fail() {
        let r = parse("", "error: permission denied", 1, "cargo flamegraph");
        assert_eq!(r.status, ToolStatus::Error);
    }

    #[test]
    fn test_flamegraph_no_svg() {
        let r = parse("Running...", "", 0, "cargo flamegraph");
        assert_eq!(r.status, ToolStatus::Ok);
    }
}
