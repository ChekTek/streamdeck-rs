use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

pub const PLUGIN_SUFFIX: &str = ".sdPlugin";

pub fn require_supported_host() -> Result<()> {
    if cfg!(target_os = "macos") || cfg!(windows) {
        Ok(())
    } else {
        bail!("Stream Deck is not supported on this platform");
    }
}

/// Installation directory Stream Deck loads plugins from.
pub fn plugins_path() -> Result<PathBuf> {
    if let Ok(override_path) = std::env::var("STREAMDECK_PLUGINS_DIR") {
        return Ok(PathBuf::from(override_path));
    }

    let home = dirs_home()?;
    if cfg!(target_os = "macos") {
        Ok(home.join("Library/Application Support/com.elgato.StreamDeck/Plugins"))
    } else if cfg!(windows) {
        let appdata = std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join("AppData/Roaming"));
        Ok(appdata.join("Elgato/StreamDeck/Plugins"))
    } else {
        bail!("Stream Deck is not supported on this platform");
    }
}

fn dirs_home() -> Result<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .context("unable to determine home directory")
}

pub struct InstalledPlugin {
    pub uuid: String,
    pub path: PathBuf,
    pub is_link: bool,
    pub target_path: Option<PathBuf>,
}

pub fn get_plugins() -> Result<Vec<InstalledPlugin>> {
    let root = plugins_path()?;
    if !root.exists() {
        return Ok(Vec::new());
    }

    let mut plugins = Vec::new();
    for entry in std::fs::read_dir(&root)? {
        let entry = entry?;
        let path = entry.path();
        let Some(uuid) = plugin_id_from_path(&path) else {
            continue;
        };
        if !path.is_dir() && !is_link(&path) {
            continue;
        }
        let is_link = is_link(&path);
        let target_path = if is_link {
            std::fs::read_link(&path).ok()
        } else {
            None
        };
        plugins.push(InstalledPlugin {
            uuid,
            path,
            is_link,
            target_path,
        });
    }
    plugins.sort_by(|a, b| a.uuid.cmp(&b.uuid));
    Ok(plugins)
}

pub fn is_plugin_installed(uuid: &str) -> Result<bool> {
    Ok(get_plugins()?.iter().any(|p| p.uuid == uuid))
}

pub fn plugin_id_from_path(path: &Path) -> Option<String> {
    let name = path.file_name()?.to_str()?;
    name.strip_suffix(PLUGIN_SUFFIX).map(str::to_string)
}

pub fn is_sdplugin_dir(path: &Path) -> bool {
    plugin_id_from_path(path).is_some() && (path.is_dir() || is_link(path))
}

pub fn is_link(path: &Path) -> bool {
    #[cfg(windows)]
    {
        junction::exists(path).unwrap_or(false)
            || path
                .symlink_metadata()
                .map(|m| m.file_type().is_symlink())
                .unwrap_or(false)
    }
    #[cfg(not(windows))]
    {
        path.symlink_metadata()
            .map(|m| m.file_type().is_symlink())
            .unwrap_or(false)
    }
}

pub fn generate_plugin_id(author: &str, name: &str) -> Option<String> {
    let author = get_safe_value(author)?;
    let name = get_safe_value(name)?;
    Some(format!("com.{author}.{name}"))
}

pub fn get_safe_value(value: &str) -> Option<String> {
    let safe: String = value
        .to_lowercase()
        .replace([' ', '_'], "-")
        .chars()
        .filter(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == '-')
        .collect();
    if safe.is_empty() { None } else { Some(safe) }
}

pub fn is_valid_plugin_id(uuid: &str) -> bool {
    let mut parts = uuid.split('.');
    let Some(first) = parts.next() else {
        return false;
    };
    if !is_uuid_section(first) {
        return false;
    }
    let mut rest = 0usize;
    for part in parts {
        if !is_uuid_section(part) {
            return false;
        }
        rest += 1;
    }
    rest >= 1
}

fn is_uuid_section(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

const INVALID_DIR_CHARS: &[char] = &['\\', '/', ':', '*', '?', '"', '<', '>', '|'];

pub fn is_safe_base_name(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty() && !trimmed.starts_with('.') && !trimmed.contains(INVALID_DIR_CHARS)
}

pub fn crate_name_from(value: &str) -> String {
    let mut name: String = value
        .to_lowercase()
        .replace([' ', '_'], "-")
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
        .collect();
    if name.is_empty() {
        name = "plugin".into();
    }
    if name.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        name = format!("plugin-{name}");
    }
    name
}

pub fn is_stream_deck_running() -> bool {
    if cfg!(target_os = "macos") {
        std::process::Command::new("pgrep")
            .args(["-if", "Stream Deck"])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    } else if cfg!(windows) {
        std::process::Command::new("tasklist")
            .args(["/FI", "IMAGENAME eq StreamDeck.exe"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).contains("StreamDeck.exe"))
            .unwrap_or(false)
    } else {
        false
    }
}

pub fn run_url(url: &str) -> Result<()> {
    open::that(url).with_context(|| format!("failed to open {url}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_plugin_id() {
        assert_eq!(
            generate_plugin_id("Elgato", "Hello World").as_deref(),
            Some("com.elgato.hello-world")
        );
    }

    #[test]
    fn validates_plugin_ids() {
        assert!(is_valid_plugin_id("com.elgato.counter"));
        assert!(is_valid_plugin_id("tv.twitch.studio"));
        assert!(!is_valid_plugin_id("counter"));
        assert!(!is_valid_plugin_id("Com.Elgato.Counter"));
        assert!(!is_valid_plugin_id("com.elgato.hello_world"));
    }

    #[test]
    fn crate_names() {
        assert_eq!(crate_name_from("hello-world"), "hello-world");
        assert_eq!(crate_name_from("123"), "plugin-123");
    }

    #[test]
    fn plugin_id_from_folder() {
        assert_eq!(
            plugin_id_from_path(Path::new("com.elgato.counter.sdPlugin")).as_deref(),
            Some("com.elgato.counter")
        );
        assert_eq!(plugin_id_from_path(Path::new("not-a-plugin")), None);
    }
}
