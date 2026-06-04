pub mod asan;
pub mod audit;
pub mod bench;
pub mod binary;
pub mod bloat;
pub mod build;
pub mod clippy;
pub mod coverage;
pub mod deny;
pub mod deps;
pub mod doc;
pub mod flamegraph;
pub mod fmt;
pub mod geiger;
pub mod metrics;
pub mod msrv;
pub mod semver;
pub mod test;
pub mod udeps;
pub mod valgrind;

use crate::report::ToolReport;

/// Dispatch to the appropriate parser based on tool name.
pub fn parse_tool_output(
    tool_name: &str,
    stdout: &str,
    stderr: &str,
    exit_code: i32,
    command: &str,
) -> ToolReport {
    match tool_name {
        "build" => build::parse(stdout, stderr, exit_code, command),
        "test" => test::parse(stdout, stderr, exit_code, command),
        "coverage" => coverage::parse(stdout, stderr, exit_code, command),
        "clippy" => clippy::parse(stdout, stderr, exit_code, command),
        "fmt" => fmt::parse(stdout, stderr, exit_code, command),
        "doc" => doc::parse(stdout, stderr, exit_code, command),
        "audit" => audit::parse(stdout, stderr, exit_code, command),
        "asan" => asan::parse(stdout, stderr, exit_code, command),
        "deny" => deny::parse(stdout, stderr, exit_code, command),
        "geiger" => geiger::parse(stdout, stderr, exit_code, command),
        "metrics" => metrics::parse(stdout, stderr, exit_code, command),
        "deps" => deps::parse(stdout, stderr, exit_code, command),
        "msrv" => msrv::parse(stdout, stderr, exit_code, command),
        "semver" => semver::parse(stdout, stderr, exit_code, command),
        "udeps" => udeps::parse(stdout, stderr, exit_code, command),
        "bench" => bench::parse(stdout, stderr, exit_code, command),
        "bloat" => bloat::parse(stdout, stderr, exit_code, command),
        "flamegraph" => flamegraph::parse(stdout, stderr, exit_code, command),
        "binary" => binary::parse(stdout, stderr, exit_code, command),
        "valgrind_memcheck" | "valgrind_helgrind" | "valgrind_drd" => {
            valgrind::parse(tool_name, stdout, stderr, exit_code, command)
        }
        _ => generic_parse(tool_name, stdout, stderr, exit_code, command),
    }
}

fn generic_parse(
    tool_name: &str,
    stdout: &str,
    stderr: &str,
    exit_code: i32,
    command: &str,
) -> ToolReport {
    use crate::config::effective_output_path;
    use crate::report::ToolStatus;

    let combined = format!("{}\n{}", stdout, stderr);
    let status = if exit_code == 0 {
        ToolStatus::Ok
    } else {
        ToolStatus::Error
    };
    let summary = if exit_code == 0 {
        "检查完成".to_string()
    } else {
        "检查失败".to_string()
    };
    let markdown_content = format!(
        "# {tool_name}\n\n**命令**: `{command}`\n\n**状态**: {}\n\n**摘要**: {}\n\n## 输出\n\n```\n{}\n```\n",
        if exit_code == 0 { "✅ 成功" } else { "❌ 失败" },
        summary,
        combined.trim()
    );

    ToolReport {
        tool_name: tool_name.to_string(),
        status,
        summary,
        output_path: effective_output_path(tool_name, None),
        markdown_content,
    }
}
