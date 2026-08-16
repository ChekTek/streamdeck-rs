use anyhow::{Result, bail};
use colored::Colorize;

use crate::stream_deck::{is_plugin_installed, is_stream_deck_running, run_url};

pub fn run(uuid: &str, no_start: bool) -> Result<()> {
    restart_plugin(uuid, no_start)
}

pub fn restart_plugin(uuid: &str, no_start: bool) -> Result<()> {
    if !is_plugin_installed(uuid)? {
        bail!("Restarting failed\nPlugin not found: {uuid}");
    }

    let url = format!("streamdeck://plugins/restart/{uuid}");
    if !is_stream_deck_running() {
        if no_start {
            return Ok(());
        }
        run_url(&url)?;
        println!("Stream Deck is not running. Starting Stream Deck.");
        return Ok(());
    }

    run_url(&url)?;
    println!("Restarted {}", uuid.green());
    Ok(())
}

pub fn run_url_stop(uuid: &str) -> Result<()> {
    let url = format!("streamdeck://plugins/stop/{uuid}");
    run_url(&url)
}
