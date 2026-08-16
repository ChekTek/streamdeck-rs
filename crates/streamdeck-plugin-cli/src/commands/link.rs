use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use colored::Colorize;

use crate::stream_deck::{get_plugins, is_valid_plugin_id, plugin_id_from_path, plugins_path};

pub fn run(path: Option<PathBuf>) -> Result<()> {
    let path = path.unwrap_or_else(|| std::env::current_dir().expect("cwd"));
    link_plugin(&path)?;
    println!("{}", "Linked successfully".green());
    Ok(())
}

pub fn link_plugin(path: &Path) -> Result<()> {
    let path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    if !path.is_dir() {
        bail!("Linking failed\nDirectory not found: {}", path.display());
    }

    let Some(uuid) = plugin_id_from_path(&path) else {
        bail!(
            "Linking failed\nInvalid directory name: {}\nName must be in reverse DNS format, be suffixed with \".sdPlugin\", and must only contain lowercase alphanumeric characters (a-z, 0-9), hyphens (-), and periods (.).\nExamples: {} {}",
            path.file_name().and_then(|n| n.to_str()).unwrap_or(""),
            "com.elgato.wave-link.sdPlugin".green(),
            "tv.twitch.studio.sdPlugin".green()
        );
    };
    if !is_valid_plugin_id(&uuid) {
        bail!("Linking failed\nInvalid plugin UUID: {uuid}");
    }

    let plugins_dir = plugins_path()?;
    std::fs::create_dir_all(&plugins_dir)
        .with_context(|| format!("failed to create {}", plugins_dir.display()))?;
    let dest = plugins_dir.join(path.file_name().expect("plugin folder name"));

    if let Some(existing) = get_plugins()?.into_iter().find(|p| p.uuid == uuid) {
        if existing
            .target_path
            .as_ref()
            .and_then(|t| t.canonicalize().ok())
            == path.canonicalize().ok()
        {
            remove_link(&existing.path)?;
        } else {
            bail!(
                "Linking failed\nPlugin already installed: {uuid}\nAnother plugin with this UUID is already installed. Please uninstall the plugin, or rename the directory being linked, and try again."
            );
        }
    }

    create_link(&path, &dest)?;
    Ok(())
}

fn create_link(source: &Path, dest: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(source, dest).with_context(|| {
            format!(
                "failed to symlink {} -> {}",
                dest.display(),
                source.display()
            )
        })
    }
    #[cfg(windows)]
    {
        junction::create(source, dest).with_context(|| {
            format!(
                "failed to create junction {} -> {}",
                dest.display(),
                source.display()
            )
        })
    }
}

fn remove_link(path: &Path) -> Result<()> {
    std::fs::remove_file(path).or_else(|_| std::fs::remove_dir(path))?;
    Ok(())
}
