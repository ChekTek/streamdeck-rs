use anyhow::Result;
use colored::Colorize;

use crate::stream_deck::get_plugins;

pub fn run(all: bool) -> Result<()> {
    for plugin in get_plugins()? {
        if !plugin.is_link && !all {
            continue;
        }
        match plugin.target_path {
            Some(target) if target.exists() => {
                println!(
                    "{} {} {}",
                    plugin.uuid,
                    "→".dimmed(),
                    target.display().to_string().green()
                );
            }
            Some(target) => {
                println!(
                    "{} {} {} {}",
                    plugin.uuid,
                    "→".dimmed(),
                    target.display().to_string().red(),
                    "(not found)".dimmed()
                );
            }
            None => println!("{}", plugin.uuid),
        }
    }
    Ok(())
}
