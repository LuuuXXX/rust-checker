use crate::cli::ReportFormat;
use crate::config::Config;
use crate::engine::dependency_check::check_tool_deps;
use anyhow::Result;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;
use std::time::Instant;

pub struct Runner {
    pub config: Config,
    pub format: ReportFormat,
    pub ci_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunSummary {
    pub tool_results: Vec<ToolRunResult>,
    pub timestamp: DateTime<Local>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CiOutput {
    pub timestamp: String,
    pub results: Vec<CiToolResult>,
    pub overall: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CiToolResult {
    pub name: String,
    pub status: String,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolRunResult {
    pub name: String,
    pub status: ToolStatus,
    pub duration_ms: u64,
    pub output: String,
    pub skipped_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ToolStatus {
    Ok,
    Warn,
    Error,
    Skipped,
}

impl RunSummary {
    pub fn to_ci_output(&self) -> CiOutput {
        let results: Vec<CiToolResult> = self
            .tool_results
            .iter()
            .map(|r| CiToolResult {
                name: r.name.clone(),
                status: format!("{:?}", r.status),
                duration_ms: r.duration_ms,
            })
            .collect();
        let overall = if self
            .tool_results
            .iter()
            .any(|r| r.status == ToolStatus::Error)
        {
            "error"
        } else if self
            .tool_results
            .iter()
            .any(|r| r.status == ToolStatus::Warn)
        {
            "warn"
        } else {
            "ok"
        };
        CiOutput {
            timestamp: self.timestamp.to_rfc3339(),
            results,
            overall: overall.to_string(),
        }
    }
}

impl Runner {
    pub fn run(&self) -> Result<RunSummary> {
        let tools = match &self.config.tools {
            Some(t) => t.clone(),
            None => {
                println!("No tools configured. Run `rust-checker init` first.");
                return Ok(RunSummary {
                    tool_results: vec![],
                    timestamp: Local::now(),
                });
            }
        };

        let ordered = topological_sort(&tools);
        let total = ordered.len();
        let mut results = Vec::new();

        for (i, name) in ordered.iter().enumerate() {
            let tool_config = match tools.get(name) {
                Some(c) => c,
                None => continue,
            };

            // Skip inactive tools
            if tool_config.active.as_deref() == Some("false") {
                results.push(ToolRunResult {
                    name: name.clone(),
                    status: ToolStatus::Skipped,
                    duration_ms: 0,
                    output: String::new(),
                    skipped_reason: Some("Tool is disabled".to_string()),
                });
                continue;
            }

            // Check dependencies
            if let Some(deps) = &tool_config.depends_on {
                let binary_deps: Vec<String> = deps
                    .iter()
                    .filter(|d| !d.starts_with("tool:") && !tools.contains_key(d.as_str()))
                    .cloned()
                    .collect();
                if !binary_deps.is_empty() {
                    let missing = check_tool_deps(name, &binary_deps);
                    if !missing.is_empty() {
                        let reason = format!(
                            "Missing deps: {}",
                            missing
                                .iter()
                                .map(|m| m.name.as_str())
                                .collect::<Vec<_>>()
                                .join(", ")
                        );
                        println!("[{}/{}] {} ... SKIPPED ({})", i + 1, total, name, reason);
                        results.push(ToolRunResult {
                            name: name.clone(),
                            status: ToolStatus::Skipped,
                            duration_ms: 0,
                            output: String::new(),
                            skipped_reason: Some(reason),
                        });
                        continue;
                    }
                }
            }

            let cmd_str = match &tool_config.input_command {
                Some(c) => c.clone(),
                None => {
                    println!("[{}/{}] {} ... SKIPPED (no command)", i + 1, total, name);
                    results.push(ToolRunResult {
                        name: name.clone(),
                        status: ToolStatus::Skipped,
                        duration_ms: 0,
                        output: String::new(),
                        skipped_reason: Some("No command configured".to_string()),
                    });
                    continue;
                }
            };

            print!("[{}/{}] {} ... ", i + 1, total, name);
            let start = Instant::now();
            let result = run_command(&cmd_str);
            let duration_ms = start.elapsed().as_millis() as u64;

            match result {
                Ok((stdout, stderr, exit_code)) => {
                    let combined = format!("{}\n{}", stdout, stderr);
                    let status = if exit_code == 0 {
                        ToolStatus::Ok
                    } else {
                        ToolStatus::Error
                    };
                    let symbol = match status {
                        ToolStatus::Ok => "✓",
                        ToolStatus::Error => "✗",
                        _ => "?",
                    };
                    println!("{} ({}ms)", symbol, duration_ms);

                    // Write report
                    if let Some(output_path) = &tool_config.output_path {
                        let _ = write_report(name, output_path, &combined, &self.format);
                    }

                    results.push(ToolRunResult {
                        name: name.clone(),
                        status,
                        duration_ms,
                        output: combined,
                        skipped_reason: None,
                    });
                }
                Err(e) => {
                    println!("ERROR ({}ms): {}", duration_ms, e);
                    results.push(ToolRunResult {
                        name: name.clone(),
                        status: ToolStatus::Error,
                        duration_ms,
                        output: e.to_string(),
                        skipped_reason: None,
                    });
                }
            }
        }

        let summary = RunSummary {
            tool_results: results,
            timestamp: Local::now(),
        };

        // Save to history
        let _ = crate::history::save_run(&summary);

        // Write summary report
        let _ = write_summary_report(&summary, &self.format);

        Ok(summary)
    }
}

fn run_command(cmd_str: &str) -> Result<(String, String, i32)> {
    let parts: Vec<&str> = cmd_str.split_whitespace().collect();
    if parts.is_empty() {
        return Err(anyhow::anyhow!("Empty command"));
    }
    let output = Command::new(parts[0]).args(&parts[1..]).output()?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let exit_code = output.status.code().unwrap_or(-1);
    Ok((stdout, stderr, exit_code))
}

fn topological_sort(tools: &HashMap<String, crate::config::ToolConfig>) -> Vec<String> {
    let mut sorted = Vec::new();
    let mut visited = std::collections::HashSet::new();

    fn visit(
        name: &str,
        tools: &HashMap<String, crate::config::ToolConfig>,
        sorted: &mut Vec<String>,
        visited: &mut std::collections::HashSet<String>,
    ) {
        if visited.contains(name) {
            return;
        }
        visited.insert(name.to_string());
        if let Some(config) = tools.get(name) {
            if let Some(deps) = &config.depends_on {
                for dep in deps {
                    if tools.contains_key(dep.as_str()) {
                        visit(dep, tools, sorted, visited);
                    }
                }
            }
        }
        sorted.push(name.to_string());
    }

    let mut names: Vec<String> = tools.keys().cloned().collect();
    names.sort(); // deterministic order
    for name in &names {
        visit(name, tools, &mut sorted, &mut visited);
    }

    sorted
}

fn write_report(
    tool_name: &str,
    output_path: &str,
    content: &str,
    format: &ReportFormat,
) -> Result<()> {
    let ext = match format {
        ReportFormat::Markdown => "md",
        ReportFormat::Html => "html",
        ReportFormat::Json => "json",
    };
    let path = std::path::PathBuf::from(format!("{}.{}", output_path, ext));
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    match format {
        ReportFormat::Markdown => {
            let md = format!("# {}\n\n```\n{}\n```\n", tool_name, content);
            std::fs::write(&path, md)?;
        }
        ReportFormat::Html => {
            let html = format!(
                "<html><body><h1>{}</h1><pre>{}</pre></body></html>",
                tool_name, content
            );
            std::fs::write(&path, html)?;
        }
        ReportFormat::Json => {
            let json = serde_json::json!({ "tool": tool_name, "output": content });
            std::fs::write(&path, serde_json::to_string_pretty(&json)?)?;
        }
    }
    Ok(())
}

fn write_summary_report(summary: &RunSummary, format: &ReportFormat) -> Result<()> {
    let dir = std::path::PathBuf::from(".localcheck/reports");
    std::fs::create_dir_all(&dir)?;

    let mut lines = vec!["# Run Summary\n".to_string()];
    lines.push(format!("Timestamp: {}\n", summary.timestamp));
    lines.push("| Tool | Status | Duration |\n|------|--------|----------|\n".to_string());
    for r in &summary.tool_results {
        let symbol = match r.status {
            ToolStatus::Ok => "✅",
            ToolStatus::Warn => "⚠️",
            ToolStatus::Error => "❌",
            ToolStatus::Skipped => "⏭️",
        };
        lines.push(format!("| {} | {} | {}ms |\n", r.name, symbol, r.duration_ms));
    }

    match format {
        ReportFormat::Markdown | ReportFormat::Html => {
            let content = lines.join("");
            std::fs::write(dir.join("summary.md"), &content)?;
        }
        ReportFormat::Json => {
            let json = serde_json::to_string_pretty(summary)?;
            std::fs::write(dir.join("summary.json"), json)?;
        }
    }
    Ok(())
}
