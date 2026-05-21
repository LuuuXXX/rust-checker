use anyhow::Result;
use notify::{Event, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::{Duration, Instant};

pub struct WatchOptions {
    pub paths: Vec<PathBuf>,
    pub debounce_ms: u64,
    pub tools: Option<Vec<String>>,
}

/// Watch `paths` for changes and call `run_fn` after each debounced event.
///
/// Returns when the watcher is interrupted (Ctrl-C or channel disconnect).
pub fn watch<F>(options: WatchOptions, mut run_fn: F) -> Result<()>
where
    F: FnMut(Option<Vec<String>>) -> Result<()>,
{
    let (tx, rx) = mpsc::channel::<notify::Result<Event>>();

    let mut watcher = notify::recommended_watcher(tx)?;

    if options.paths.is_empty() {
        anyhow::bail!(
            "没有配置监听路径。请在 config.toml 的 [watch] 段中设置 paths，例如：\n\
             \n\
             [watch]\n\
             paths = [\"src\"]\n"
        );
    }

    for path in &options.paths {
        if path.exists() {
            watcher.watch(path, RecursiveMode::Recursive)?;
        } else {
            eprintln!("⚠️  监听路径不存在，跳过: {}", path.display());
        }
    }

    let debounce = Duration::from_millis(options.debounce_ms);

    println!(
        "👀 监听文件变更 ({} 个路径, 防抖 {}ms)... 按 Ctrl-C 退出",
        options.paths.len(),
        options.debounce_ms
    );

    let mut last_event: Option<Instant> = None;

    loop {
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(Ok(_event)) => {
                last_event = Some(Instant::now());
            }
            Ok(Err(e)) => {
                eprintln!("⚠️  监听错误: {e}");
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }

        // Trigger run after debounce window has elapsed
        if let Some(t) = last_event {
            if t.elapsed() >= debounce {
                last_event = None;
                println!("\n🔄 检测到文件变更，重新运行检查...\n");
                if let Err(e) = run_fn(options.tools.clone()) {
                    eprintln!("⚠️  运行失败: {e}");
                }
            }
        }
    }

    Ok(())
}

/// Build `WatchOptions` from a config watch section and a project directory.
pub fn build_options(
    watch_cfg: Option<&crate::config::WatchConfig>,
    project_dir: &Path,
) -> WatchOptions {
    let (paths, debounce_ms, tools) = match watch_cfg {
        Some(cfg) => {
            let configured_paths: Vec<PathBuf> = cfg
                .paths
                .as_deref()
                .unwrap_or(&[])
                .iter()
                .map(|p| project_dir.join(p))
                .collect();
            // Fall back to the same default as the None branch to avoid an opaque error
            // in watch() when [watch] is present but paths is absent or explicitly empty.
            let paths = if configured_paths.is_empty() {
                let src = project_dir.join("src");
                let default_path = if src.exists() {
                    src
                } else {
                    project_dir.to_path_buf()
                };
                vec![default_path]
            } else {
                configured_paths
            };
            let debounce_ms = cfg.debounce_ms();
            let tools = cfg.tools.clone();
            (paths, debounce_ms, tools)
        }
        None => {
            // Default: watch `src/` if present, else project root
            let src = project_dir.join("src");
            let default_path = if src.exists() {
                src
            } else {
                project_dir.to_path_buf()
            };
            (vec![default_path], 500, None)
        }
    };

    WatchOptions {
        paths,
        debounce_ms,
        tools,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_options_default() {
        let dir = tempfile::tempdir().unwrap();
        let opts = build_options(None, dir.path());
        assert_eq!(opts.debounce_ms, 500);
        assert!(opts.tools.is_none());
    }

    #[test]
    fn test_build_options_with_config_no_paths_falls_back_to_default() {
        // [watch] section present but paths = None → should fall back to src/ (or project root)
        let dir = tempfile::tempdir().unwrap();
        let cfg = crate::config::WatchConfig {
            paths: None,
            debounce_ms: Some(200),
            tools: None,
        };
        let opts = build_options(Some(&cfg), dir.path());
        // paths must not be empty (would cause bail! in watch())
        assert!(
            !opts.paths.is_empty(),
            "paths should not be empty when not configured"
        );
        assert_eq!(opts.debounce_ms, 200);
    }

    #[test]
    fn test_build_options_with_config_empty_paths_falls_back_to_default() {
        // [watch] section present with explicit empty paths → should fall back to src/ (or project root)
        let dir = tempfile::tempdir().unwrap();
        let cfg = crate::config::WatchConfig {
            paths: Some(vec![]),
            debounce_ms: None,
            tools: None,
        };
        let opts = build_options(Some(&cfg), dir.path());
        assert!(
            !opts.paths.is_empty(),
            "paths should not be empty when configured as []"
        );
    }

    #[test]
    fn test_build_options_with_config() {
        let dir = tempfile::tempdir().unwrap();
        let cfg = crate::config::WatchConfig {
            paths: Some(vec!["src".to_string()]),
            debounce_ms: Some(300),
            tools: Some(vec!["clippy".to_string()]),
        };
        let opts = build_options(Some(&cfg), dir.path());
        assert_eq!(opts.debounce_ms, 300);
        assert_eq!(
            opts.tools.as_deref(),
            Some(["clippy".to_string()].as_slice())
        );
        assert_eq!(opts.paths, vec![dir.path().join("src")]);
    }
}
