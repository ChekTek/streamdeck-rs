use std::path::PathBuf;

use anyhow::{Result, bail};
use colored::Colorize;
use inquire::{Confirm, Text, validator::Validation};

use crate::project::{cargo_build_release, copy_release_binary};
use crate::stream_deck::{
    crate_name_from, generate_plugin_id, get_plugins, is_safe_base_name, is_valid_plugin_id,
    require_supported_host,
};
use crate::template::{PluginInfo, render_template};

use super::dev;
use super::link;
use super::restart;

pub fn run() -> Result<()> {
    require_supported_host()?;
    println!(
        "Welcome to the {} creation wizard.",
        "Stream Deck Plugin".green()
    );
    println!();
    println!(
        "This utility will guide you through creating a local development environment for a plugin."
    );
    println!(
        "For more information on building plugins see {}.",
        "https://docs.elgato.com".blue()
    );
    println!();
    println!("{}", "Press ^C at any time to quit.".dimmed());
    println!();

    let info = prompt_plugin_info()?;
    let destination = validate_destination(&info.uuid)?;

    println!();
    if !Confirm::new("Create Stream Deck plugin from information above?")
        .with_default(true)
        .prompt()?
    {
        println!("Aborted");
        return Ok(());
    }

    if get_plugins()?.iter().any(|p| p.uuid == info.uuid) {
        bail!(
            "another plugin with this UUID is already installed: {}",
            info.uuid
        );
    }
    if destination.exists() {
        bail!("directory already exists: {}", destination.display());
    }

    let crate_name = crate_name_from(
        destination
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&info.uuid),
    );
    let info = PluginInfo { crate_name, ..info };

    println!();
    println!("Creating {}...", info.name.blue());

    spin("Enabling developer mode", || dev::set_developer_mode(false))?;
    spin("Generating plugin", || render_template(&destination, &info))?;

    let manifest = destination.join("Cargo.toml");
    let plugin_dir = destination.join(format!("{}.sdPlugin", info.uuid));
    spin("Building plugin", || cargo_build_release(&manifest))?;
    spin("Copying binary", || {
        copy_release_binary(&manifest, &plugin_dir).map(|_| ())
    })?;
    spin("Finalizing setup", || {
        link::link_plugin(&plugin_dir)?;
        restart::restart_plugin(&info.uuid, true)
    })?;

    println!();
    println!("{}", "Successfully created plugin!".green());
    try_open_editor(&destination)?;
    Ok(())
}

fn prompt_plugin_info() -> Result<PluginInfo> {
    let required = |message: &'static str| {
        move |value: &str| {
            if value.trim().is_empty() {
                Ok(Validation::Invalid(message.into()))
            } else {
                Ok(Validation::Valid)
            }
        }
    };

    let author = Text::new("Author:")
        .with_validator(required("Please enter the author."))
        .prompt()?;
    let name = Text::new("Plugin Name:")
        .with_validator(required("Please enter the name of the plugin."))
        .prompt()?;

    let default_uuid = generate_plugin_id(&author, &name).unwrap_or_default();
    let mut uuid_prompt = Text::new("Plugin UUID:").with_validator(|uuid: &str| {
        if !is_valid_plugin_id(uuid) {
            return Ok(Validation::Invalid(
                "UUID must be in reverse DNS format, and must only contain lowercase alphanumeric characters (a-z, 0-9), hyphens (-), and periods (.).".into(),
            ));
        }
        if get_plugins()
            .map(|plugins| plugins.iter().any(|p| p.uuid == uuid))
            .unwrap_or(false)
        {
            return Ok(Validation::Invalid(
                "Another plugin with this UUID is already installed.".into(),
            ));
        }
        Ok(Validation::Valid)
    });
    if !default_uuid.is_empty() {
        uuid_prompt = uuid_prompt.with_default(&default_uuid);
    }
    let uuid = uuid_prompt.prompt()?;

    let description = Text::new("Description:")
        .with_validator(required(
            "Please enter a brief description of what the plugin will do.",
        ))
        .prompt()?;

    Ok(PluginInfo {
        author,
        name,
        uuid,
        description,
        crate_name: String::new(),
    })
}

fn default_directory_name(uuid: &str) -> &str {
    uuid.rsplit('.').next().unwrap_or("")
}

fn validate_destination(uuid: &str) -> Result<PathBuf> {
    let default = default_directory_name(uuid).to_string();
    let cwd = std::env::current_dir()?;
    let candidate = cwd.join(&default);
    if is_safe_base_name(&default) && !candidate.exists() {
        return Ok(candidate);
    }

    let cwd_for_validator = cwd.clone();
    let dir = Text::new("Directory:")
        .with_default(&default)
        .with_validator(move |value: &str| {
            if !is_safe_base_name(value) {
                return Ok(Validation::Invalid("Directory name is invalid.".into()));
            }
            if cwd_for_validator.join(value).exists() {
                return Ok(Validation::Invalid("Directory already exists.".into()));
            }
            Ok(Validation::Valid)
        })
        .prompt()?;
    Ok(cwd.join(dir))
}

fn spin(message: &str, work: impl FnOnce() -> Result<()>) -> Result<()> {
    let pb = indicatif::ProgressBar::new_spinner();
    pb.set_message(message.to_string());
    pb.enable_steady_tick(std::time::Duration::from_millis(80));
    match work() {
        Ok(()) => {
            pb.finish_with_message(message.to_string());
            Ok(())
        }
        Err(err) => {
            pb.finish_and_clear();
            Err(err)
        }
    }
}

fn try_open_editor(destination: &std::path::Path) -> Result<()> {
    let editor = if which::which("cursor").is_ok() {
        Some(("cursor", "Cursor"))
    } else if which::which("code").is_ok() {
        Some(("code", "VS Code"))
    } else {
        None
    };

    let Some((bin, label)) = editor else {
        return Ok(());
    };

    println!();
    if Confirm::new(&format!("Would you like to open the plugin in {label}?"))
        .with_default(true)
        .prompt()?
    {
        let _ = std::process::Command::new(bin)
            .args(["./", "--goto", "src/main.rs"])
            .current_dir(destination)
            .status();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_directory_uses_last_uuid_segment() {
        assert_eq!(default_directory_name("com.elgato.counter"), "counter");
        assert_eq!(default_directory_name("tv.twitch"), "twitch");
        assert_eq!(default_directory_name("com.example.my.plugin"), "plugin");
    }
}
