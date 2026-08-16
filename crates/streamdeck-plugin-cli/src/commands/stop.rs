use anyhow::{Result, bail};
use colored::Colorize;

use crate::stream_deck::{is_plugin_installed, is_stream_deck_running};

use super::restart::run_url_stop;

pub fn run(uuid: &str) -> Result<()> {
    if !is_plugin_installed(uuid)? {
        bail!("Stopping failed\nPlugin not found: {uuid}");
    }

    if !is_stream_deck_running() {
        println!("Stream Deck is not running.");
        return Ok(());
    }

    run_url_stop(uuid)?;
    println!("Stopped {}", uuid.green());
    Ok(())
}
