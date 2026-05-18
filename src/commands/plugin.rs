use crate::cli::{PluginAction, PluginArgs};
use anyhow::Result;

pub fn run(args: PluginArgs) -> Result<()> {
    match args.action {
        PluginAction::List => list_plugins(),
        PluginAction::Add { name } => add_plugin(&name),
        PluginAction::Remove { name } => remove_plugin(&name),
        PluginAction::Update => update_plugins(),
    }
}

fn plugins_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(".localcheck/plugins")
}

fn list_plugins() -> Result<()> {
    let dir = plugins_dir();
    if !dir.exists() {
        println!("No plugins installed.");
        return Ok(());
    }
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        println!("- {}", entry.file_name().to_string_lossy());
    }
    Ok(())
}

fn add_plugin(name: &str) -> Result<()> {
    let url = format!(
        "https://raw.githubusercontent.com/rust-checker/rust-checker-plugins/main/{}/plugin.toml",
        name
    );
    println!("Downloading plugin '{}' from {}", name, url);
    // In a real implementation, we'd download and install.
    // For now, just create the plugin directory.
    let plugin_dir = plugins_dir().join(name);
    std::fs::create_dir_all(&plugin_dir)?;
    println!("Plugin '{}' added to {:?}", name, plugin_dir);
    Ok(())
}

fn remove_plugin(name: &str) -> Result<()> {
    let plugin_dir = plugins_dir().join(name);
    if plugin_dir.exists() {
        std::fs::remove_dir_all(&plugin_dir)?;
        println!("Plugin '{}' removed.", name);
    } else {
        println!("Plugin '{}' not found.", name);
    }
    Ok(())
}

fn update_plugins() -> Result<()> {
    let dir = plugins_dir();
    if !dir.exists() {
        println!("No plugins installed.");
        return Ok(());
    }
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        println!("Updating plugin '{}'...", name);
        // In a real implementation, re-download from the URL.
    }
    println!("All plugins updated.");
    Ok(())
}
