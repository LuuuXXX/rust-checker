use anyhow::Result;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use chrono::Local;

pub struct Logger {
    file: File,
    #[allow(dead_code)]
    pub log_path: PathBuf,
}

impl Logger {
    pub fn new(log_dir: &Path) -> Result<Self> {
        std::fs::create_dir_all(log_dir)?;
        let timestamp = Local::now().format("%Y%m%d-%H%M%S");
        let log_path = log_dir.join(format!("{}.log", timestamp));
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)?;
        Ok(Logger { file, log_path })
    }

    fn write_line(&mut self, line: &str) -> Result<()> {
        let ts = Local::now().format("%Y-%m-%d %H:%M:%S");
        writeln!(self.file, "[{}] {}", ts, line)?;
        Ok(())
    }

    pub fn log_env_info(&mut self) -> Result<()> {
        self.write_line(&format!("OS: {}", std::env::consts::OS))?;
        let rust_version = std::process::Command::new("rustc")
            .arg("--version")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_else(|| "unknown".to_string());
        self.write_line(&format!("Rust: {}", rust_version.trim()))?;
        Ok(())
    }

    pub fn log_tool_start(&mut self, name: &str) -> Result<()> {
        self.write_line(&format!("START tool={}", name))
    }

    pub fn log_tool_end(&mut self, name: &str, duration_secs: f64) -> Result<()> {
        self.write_line(&format!("END tool={} duration={:.2}s", name, duration_secs))
    }

    pub fn log_tool_skipped(&mut self, name: &str, reason: &str) -> Result<()> {
        self.write_line(&format!("SKIP tool={} reason={}", name, reason))
    }

    pub fn log_tool_output(&mut self, name: &str, stdout: &str, stderr: &str) -> Result<()> {
        self.write_line(&format!("OUTPUT tool={}", name))?;
        if !stdout.is_empty() {
            self.write_line(&format!(
                "  stdout: {}",
                stdout.lines().take(5).collect::<Vec<_>>().join(" | ")
            ))?;
        }
        if !stderr.is_empty() {
            self.write_line(&format!(
                "  stderr: {}",
                stderr.lines().take(5).collect::<Vec<_>>().join(" | ")
            ))?;
        }
        Ok(())
    }

    pub fn log_report_generated(&mut self, path: &str) -> Result<()> {
        self.write_line(&format!("REPORT generated={}", path))
    }
}
