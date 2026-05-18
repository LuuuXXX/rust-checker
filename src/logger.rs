use anyhow::Result;
use std::path::PathBuf;

pub struct EnvInfo {
    pub rust_version: String,
    pub os: String,
    pub cwd: String,
}

pub fn init_logger(_log_path: &PathBuf) -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .init();
    Ok(())
}

pub fn write_log_header(log_path: &PathBuf, env_info: &EnvInfo) -> Result<()> {
    use std::io::Write;
    if let Some(parent) = log_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)?;
    writeln!(file, "=== rust-checker log ===")?;
    writeln!(file, "Rust version: {}", env_info.rust_version)?;
    writeln!(file, "OS: {}", env_info.os)?;
    writeln!(file, "CWD: {}", env_info.cwd)?;
    writeln!(file, "========================")?;
    Ok(())
}
