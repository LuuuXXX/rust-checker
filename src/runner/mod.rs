pub mod dependency_check;

use anyhow::{bail, Context, Result};
use indexmap::IndexMap;
use indicatif::{ProgressBar, ProgressStyle};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::config::{Config, HistoryConfig, ToolConfig};
use crate::logger::Logger;
use crate::report::{write_summary, ReportFormat, ToolReport, ToolStatus};
use crate::runner::dependency_check::{check_tool_available, get_tool_dep, prompt_and_install};
use crate::tools::parse_tool_output;

pub struct Runner {
    config: Config,
    working_dir: PathBuf,
    report_dir: PathBuf,
    log_dir: PathBuf,
    history_dir: PathBuf,
    format: ReportFormat,
    ci_mode: bool,
}

impl Runner {
    pub fn new(
        config: Config,
        project_dir: &Path,
        working_dir: &Path,
        format: ReportFormat,
        ci_mode: bool,
    ) -> Self {
        Runner {
            config,
            working_dir: working_dir.to_path_buf(),
            report_dir: project_dir.join(".rust-checker").join("reports"),
            log_dir: project_dir.join(".rust-checker").join("logs"),
            history_dir: project_dir.join(".rust-checker").join("history"),
            format,
            ci_mode,
        }
    }

    /// Write a tool report (always markdown; also HTML when the run format is HTML).
    fn write_report(&self, report: &ToolReport) -> Result<()> {
        crate::report::markdown::write_tool_report(&self.report_dir, report)?;
        if self.format == ReportFormat::Html {
            crate::report::html::write_tool_report_html(&self.report_dir, report)?;
        }
        Ok(())
    }

    pub fn run(&self) -> Result<()> {
        std::fs::create_dir_all(&self.report_dir)?;
        std::fs::create_dir_all(&self.log_dir)?;

        let mut logger = Logger::new(&self.log_dir)?;
        logger.log_env_info()?;

        let order = topo_sort(&self.config.tools)?;

        let pb = ProgressBar::new(order.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")
                .unwrap_or_else(|_| ProgressStyle::default_bar()),
        );

        let mut reports: Vec<ToolReport> = Vec::new();

        for tool_name in &order {
            let tool_cfg = match self.config.tools.get(tool_name) {
                Some(c) => c,
                None => continue,
            };

            pb.set_message(format!("Running {}", tool_name));

            if !tool_cfg.active {
                logger.log_tool_skipped(tool_name, "inactive")?;
                let report = make_skipped_report(tool_name, tool_cfg, "工具已禁用");
                self.write_report(&report)?;
                reports.push(report);
                pb.inc(1);
                continue;
            }

            // Check if a dependency failed
            if let Some(deps) = &tool_cfg.depends_on {
                let failed_deps: Vec<_> = deps
                    .iter()
                    .filter(|d| {
                        self.config
                            .tools
                            .get(d.as_str())
                            .map(|tc| tc.active)
                            .unwrap_or(false)
                            && reports
                                .iter()
                                .any(|r| &r.tool_name == *d && r.status == ToolStatus::Error)
                    })
                    .cloned()
                    .collect();

                if !failed_deps.is_empty() {
                    let reason = format!("依赖失败: {}", failed_deps.join(", "));
                    logger.log_tool_skipped(tool_name, &reason)?;
                    let report = make_skipped_report(tool_name, tool_cfg, &reason);
                    self.write_report(&report)?;
                    reports.push(report);
                    pb.inc(1);
                    continue;
                }
            }

            // Binary dependency check
            if !check_tool_available(tool_name) {
                if let Some(dep) = get_tool_dep(tool_name) {
                    let installed = if !self.ci_mode {
                        prompt_and_install(tool_name, &dep).unwrap_or(false)
                    } else {
                        false
                    };
                    if !installed {
                        let reason = format!("缺少依赖: {}", dep.binary);
                        logger.log_tool_skipped(tool_name, &reason)?;
                        let report = make_skipped_report(tool_name, tool_cfg, &reason);
                        self.write_report(&report)?;
                        reports.push(report);
                        pb.inc(1);
                        continue;
                    }
                }
            }

            logger.log_tool_start(tool_name)?;
            let start = Instant::now();

            let cmd_str = &tool_cfg.input_command;
            let parts: Vec<&str> = cmd_str.split_whitespace().collect();
            let (program, args) = parts.split_first().unwrap_or((&"cargo", &[]));

            let output = std::process::Command::new(program)
                .args(args)
                // Run tools in the effective project directory (workspace member path
                // when --crate is used, otherwise the project root).
                .current_dir(&self.working_dir)
                .output()
                .with_context(|| format!("Failed to run: {cmd_str}"))?;

            let duration = start.elapsed().as_secs_f64();
            logger.log_tool_end(tool_name, duration)?;

            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let exit_code = output.status.code().unwrap_or(-1);

            logger.log_tool_output(tool_name, &stdout, &stderr)?;

            let mut report = parse_tool_output(tool_name, &stdout, &stderr, exit_code, cmd_str);

            // Override output_path from config if set
            if let Some(op) = &tool_cfg.output_path {
                report.output_path = op.clone();
            }

            self.write_report(&report)?;

            logger.log_report_generated(&report.output_path)?;
            reports.push(report);
            pb.inc(1);
        }

        pb.finish_with_message("Done");

        write_summary(&self.report_dir, &reports)?;

        // Also write HTML summary when format is HTML
        if self.format == ReportFormat::Html {
            crate::report::html::write_summary_html(&self.report_dir, &reports)?;
        }

        if self.ci_mode || self.format == ReportFormat::Json {
            let timestamp = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
            let json = crate::report::json::build_ci_json(&reports, &timestamp);
            let json_path = self.report_dir.join("ci_result.json");
            std::fs::write(&json_path, serde_json::to_string_pretty(&json)?)?;
            println!("JSON: {}", json_path.display());
        }

        // Persist history snapshot
        let max_entries = self
            .config
            .history
            .as_ref()
            .map(HistoryConfig::max_entries)
            .unwrap_or(10);
        if let Err(e) = crate::history::save_history(&reports, &self.history_dir, max_entries) {
            eprintln!("⚠️  历史记录保存失败: {e}");
        }

        println!("\n✅ 检查完成，报告已生成: {}", self.report_dir.display());
        Ok(())
    }
}

fn make_skipped_report(tool_name: &str, tool_cfg: &ToolConfig, reason: &str) -> ToolReport {
    let output_path = tool_cfg
        .output_path
        .clone()
        .unwrap_or_else(|| crate::config::effective_output_path(tool_name, None));
    ToolReport {
        tool_name: tool_name.to_string(),
        status: ToolStatus::Skipped,
        summary: reason.to_string(),
        output_path,
        markdown_content: format!("# {tool_name}\n\n{reason}\n"),
    }
}

fn topo_sort(tools: &IndexMap<String, ToolConfig>) -> Result<Vec<String>> {
    let mut order = Vec::new();
    let mut visited = HashSet::new();
    let mut in_stack = HashSet::new();

    let names: Vec<String> = tools.keys().cloned().collect();
    for name in &names {
        dfs(name, tools, &mut order, &mut visited, &mut in_stack)?;
    }

    Ok(order)
}

fn dfs(
    name: &str,
    tools: &IndexMap<String, ToolConfig>,
    order: &mut Vec<String>,
    visited: &mut HashSet<String>,
    in_stack: &mut HashSet<String>,
) -> Result<()> {
    if in_stack.contains(name) {
        bail!("Circular dependency detected for tool: {}", name);
    }
    if visited.contains(name) {
        return Ok(());
    }
    in_stack.insert(name.to_string());
    if let Some(cfg) = tools.get(name) {
        if let Some(deps) = &cfg.depends_on {
            for dep in deps {
                if tools.contains_key(dep.as_str()) {
                    dfs(dep, tools, order, visited, in_stack)?;
                }
            }
        }
    }
    in_stack.remove(name);
    visited.insert(name.to_string());
    order.push(name.to_string());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ToolConfig;
    use crate::report::ToolStatus;

    fn make_tool(active: bool, deps: Option<Vec<&str>>) -> ToolConfig {
        ToolConfig {
            desc: "test".to_string(),
            active,
            input_command: "cargo test".to_string(),
            output_path: None,
            depends_on: deps.map(|d| d.iter().map(|s| s.to_string()).collect()),
        }
    }

    fn make_tool_with_output(active: bool, output_path: Option<&str>) -> ToolConfig {
        ToolConfig {
            desc: "test".to_string(),
            active,
            input_command: "cargo test".to_string(),
            output_path: output_path.map(|s| s.to_string()),
            depends_on: None,
        }
    }

    #[test]
    fn test_topo_sort_no_deps() {
        let mut tools = IndexMap::new();
        tools.insert("build".to_string(), make_tool(true, None));
        tools.insert("test".to_string(), make_tool(true, None));
        let order = topo_sort(&tools).unwrap();
        assert_eq!(order.len(), 2);
    }

    #[test]
    fn test_topo_sort_with_deps() {
        let mut tools = IndexMap::new();
        tools.insert("test".to_string(), make_tool(true, Some(vec!["build"])));
        tools.insert("build".to_string(), make_tool(true, None));
        let order = topo_sort(&tools).unwrap();
        let build_pos = order.iter().position(|x| x == "build").unwrap();
        let test_pos = order.iter().position(|x| x == "test").unwrap();
        assert!(build_pos < test_pos);
    }

    #[test]
    fn test_topo_sort_chain() {
        // a → b → c (c must run first, then b, then a)
        let mut tools = IndexMap::new();
        tools.insert("a".to_string(), make_tool(true, Some(vec!["b"])));
        tools.insert("b".to_string(), make_tool(true, Some(vec!["c"])));
        tools.insert("c".to_string(), make_tool(true, None));
        let order = topo_sort(&tools).unwrap();
        let pos_c = order.iter().position(|x| x == "c").unwrap();
        let pos_b = order.iter().position(|x| x == "b").unwrap();
        let pos_a = order.iter().position(|x| x == "a").unwrap();
        assert!(pos_c < pos_b);
        assert!(pos_b < pos_a);
    }

    #[test]
    fn test_topo_sort_circular_dep() {
        let mut tools = IndexMap::new();
        tools.insert("a".to_string(), make_tool(true, Some(vec!["b"])));
        tools.insert("b".to_string(), make_tool(true, Some(vec!["a"])));
        assert!(topo_sort(&tools).is_err());
    }

    #[test]
    fn test_topo_sort_empty() {
        let tools = IndexMap::new();
        let order = topo_sort(&tools).unwrap();
        assert!(order.is_empty());
    }

    #[test]
    fn test_topo_sort_single_tool() {
        let mut tools = IndexMap::new();
        tools.insert("build".to_string(), make_tool(true, None));
        let order = topo_sort(&tools).unwrap();
        assert_eq!(order, vec!["build"]);
    }

    #[test]
    fn test_topo_sort_dep_not_in_tools_is_ignored() {
        // test depends on "build" but build is not in the tools map
        let mut tools = IndexMap::new();
        tools.insert("test".to_string(), make_tool(true, Some(vec!["build"])));
        // Should not fail - missing deps are silently ignored
        let order = topo_sort(&tools).unwrap();
        assert_eq!(order, vec!["test"]);
    }

    #[test]
    fn test_make_skipped_report_with_output_path() {
        let tool = make_tool_with_output(true, Some("quality/build.md"));
        let r = make_skipped_report("build", &tool, "test reason");
        assert_eq!(r.status, ToolStatus::Skipped);
        assert_eq!(r.output_path, "quality/build.md");
        assert_eq!(r.tool_name, "build");
        assert!(r.summary.contains("test reason"));
    }

    #[test]
    fn test_make_skipped_report_without_output_path_uses_default() {
        let tool = make_tool_with_output(true, None);
        let r = make_skipped_report("clippy", &tool, "missing dep");
        assert_eq!(r.status, ToolStatus::Skipped);
        // Should use the builtin default for clippy
        assert_eq!(r.output_path, "quality/clippy.md");
    }
}
