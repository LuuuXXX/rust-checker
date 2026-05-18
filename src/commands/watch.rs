use crate::cli::WatchArgs;
use anyhow::Result;
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::sync::mpsc::channel;
use std::time::Duration;

pub fn run(args: WatchArgs) -> Result<()> {
    let config = crate::config::load_config()?;
    let debounce_ms = config
        .watch
        .as_ref()
        .and_then(|w| w.debounce_ms)
        .unwrap_or(500);

    let tools_filter = args.tools.clone();

    println!(
        "Watching for changes (debounce: {}ms). Press Ctrl+C to stop.",
        debounce_ms
    );

    let (tx, rx) = channel::<notify::Result<Event>>();
    let mut watcher = RecommendedWatcher::new(tx, notify::Config::default())?;
    watcher.watch(std::path::Path::new("src"), RecursiveMode::Recursive)?;

    let debounce = Duration::from_millis(debounce_ms);
    loop {
        match rx.recv_timeout(debounce) {
            Ok(Ok(event)) => {
                println!("Change detected: {:?}", event.kind);
                trigger_run(&tools_filter)?;
            }
            Ok(Err(e)) => eprintln!("Watch error: {:?}", e),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
            Err(e) => {
                eprintln!("Channel error: {:?}", e);
                break;
            }
        }
    }

    Ok(())
}

fn trigger_run(tools_filter: &Option<Vec<String>>) -> Result<()> {
    println!("Running checks...");
    if let Some(tools) = tools_filter {
        println!("Running tools: {:?}", tools);
    }
    let config = crate::config::load_config()?;
    let runner = crate::engine::runner::Runner {
        config,
        format: crate::cli::ReportFormat::Markdown,
        ci_mode: false,
    };
    let _ = runner.run()?;
    Ok(())
}
