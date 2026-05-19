use anyhow::Result;
use std::path::Path;

use crate::plugin::{install_plugin, list_plugins, remove_plugin, update_plugins};

pub fn run_plugin_list(project_dir: &Path) -> Result<()> {
    let plugins = list_plugins(project_dir)?;
    if plugins.is_empty() {
        println!("没有已安装的插件。");
        println!("使用 `rust-checker plugin add <name>` 从注册表安装插件。");
        return Ok(());
    }

    println!("已安装的插件 ({}):\n", plugins.len());
    println!("{:<20} {:<10} {:<12} 描述", "名称", "版本", "分类");
    println!("{}", "-".repeat(70));
    for p in &plugins {
        println!(
            "{:<20} {:<10} {:<12} {}",
            p.plugin.name, p.plugin.version, p.plugin.category, p.plugin.description
        );
    }
    Ok(())
}

pub fn run_plugin_add(name: &str, project_dir: &Path) -> Result<()> {
    install_plugin(name, project_dir)
}

pub fn run_plugin_remove(name: &str, project_dir: &Path) -> Result<()> {
    remove_plugin(name, project_dir)
}

pub fn run_plugin_update(project_dir: &Path) -> Result<()> {
    update_plugins(project_dir)
}
