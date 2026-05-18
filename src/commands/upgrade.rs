use anyhow::Result;

pub fn run() -> Result<()> {
    let mut config = crate::config::load_config()?;

    // Backup current config
    let backup_path = std::path::PathBuf::from(".localcheck/config.toml.bak");
    if crate::config::config_path().exists() {
        std::fs::copy(crate::config::config_path(), &backup_path)?;
        println!("Backed up config to {:?}", backup_path);
    }

    // Upgrade schema version
    config.schema_version = Some(1);
    crate::config::save_config(&config)?;
    println!("Config upgraded to schema version 1.");
    Ok(())
}
