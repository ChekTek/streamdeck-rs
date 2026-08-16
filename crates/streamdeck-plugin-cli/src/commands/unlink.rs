use anyhow::{Result, bail};
use colored::Colorize;

use crate::stream_deck::get_plugins;

use super::stop;

pub fn run(uuid: &str, delete: bool) -> Result<()> {
    let plugin = get_plugins()?
        .into_iter()
        .find(|p| p.uuid == uuid)
        .ok_or_else(|| anyhow::anyhow!("Plugin not found\nNo plugin found with UUID: {uuid}"))?;

    if !plugin.is_link {
        if !delete {
            bail!(
                "Plugin cannot be unlinked\n{uuid} is not a linked plugin\nTo uninstall and delete the plugin, re-run with the delete flag (-d|--delete)"
            );
        }
        let _ = stop::run(uuid);
        std::fs::remove_dir_all(&plugin.path)?;
        println!("{}", "Uninstalled successfully".green());
        return Ok(());
    }

    let _ = stop::run(uuid);
    std::fs::remove_file(&plugin.path).or_else(|_| std::fs::remove_dir(&plugin.path))?;
    println!("{}", "Unlinked successfully".green());
    Ok(())
}
