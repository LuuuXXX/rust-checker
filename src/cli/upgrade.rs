use anyhow::Result;
use std::path::Path;

use crate::upgrade::upgrade_config;

pub fn run_upgrade(project_dir: &Path) -> Result<()> {
    upgrade_config(project_dir)
}
