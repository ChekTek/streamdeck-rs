use std::path::{Component, Path, PathBuf};

use anyhow::{Context, Result, bail};
use colored::Colorize;
use serde_json::Value;

use crate::project::resolve_project;
use crate::stream_deck::{is_valid_plugin_id, plugin_id_from_path, require_supported_host};

pub fn run(path: Option<PathBuf>) -> Result<()> {
    require_supported_host()?;
    let project = resolve_project(path)?;
    let errors = validate_plugin(&project.plugin_dir)?;
    if errors.is_empty() {
        println!("{}", "Validation successful".green());
        Ok(())
    } else {
        for error in &errors {
            println!("{}", error.red());
        }
        bail!("validation failed with {} error(s)", errors.len());
    }
}

pub fn validate_plugin(plugin_dir: &Path) -> Result<Vec<String>> {
    validate_plugin_inner(plugin_dir, false)
}

pub fn validate_plugin_for_pack(plugin_dir: &Path) -> Result<Vec<String>> {
    validate_plugin_inner(plugin_dir, true)
}

fn validate_plugin_inner(plugin_dir: &Path, require_all_os: bool) -> Result<Vec<String>> {
    let mut errors = Vec::new();
    let Some(uuid) = plugin_id_from_path(plugin_dir) else {
        errors.push(format!(
            "directory name must end with .sdPlugin: {}",
            plugin_dir.display()
        ));
        return Ok(errors);
    };
    if !is_valid_plugin_id(&uuid) {
        errors.push(format!(
            "invalid plugin UUID derived from folder name: {uuid}"
        ));
    }

    let manifest_path = plugin_dir.join("manifest.json");
    if !manifest_path.is_file() {
        errors.push("manifest.json is missing".into());
        return Ok(errors);
    }

    let raw = std::fs::read_to_string(&manifest_path).context("failed to read manifest.json")?;
    let manifest: Value = match serde_json::from_str(&raw) {
        Ok(value) => value,
        Err(err) => {
            errors.push(format!("manifest.json is not valid JSON: {err}"));
            return Ok(errors);
        }
    };

    match manifest.get("UUID").and_then(Value::as_str) {
        Some(manifest_uuid) if manifest_uuid == uuid => {}
        Some(manifest_uuid) => errors.push(format!(
            "manifest UUID `{manifest_uuid}` does not match folder `{uuid}`"
        )),
        None => errors.push("manifest.json is missing UUID".into()),
    }

    match manifest.get("Version").and_then(Value::as_str) {
        Some(version) if is_valid_plugin_version(version) => {}
        Some(version) => errors.push(format!(
            "invalid plugin version `{version}`: expected 1 to 4 numeric components"
        )),
        None => errors.push("manifest.json is missing Version".into()),
    }

    check_icon(plugin_dir, &manifest, "Icon", &mut errors);
    check_icon(plugin_dir, &manifest, "CategoryIcon", &mut errors);
    check_code_paths(plugin_dir, &manifest, require_all_os, &mut errors);

    if let Some(actions) = manifest.get("Actions").and_then(Value::as_array) {
        for action in actions {
            check_icon_value(
                plugin_dir,
                action.get("Icon").and_then(Value::as_str),
                "action Icon",
                &mut errors,
            );
            if let Some(pi) = action.get("PropertyInspectorPath").and_then(Value::as_str) {
                match existing_file_in_plugin(plugin_dir, pi) {
                    Ok(true) => {}
                    Ok(false) => errors.push(format!("missing property inspector: {pi}")),
                    Err(()) => errors.push(format!(
                        "property inspector path is outside the plugin directory: {pi}"
                    )),
                }
            }
            if let Some(states) = action.get("States").and_then(Value::as_array) {
                for state in states {
                    check_icon_value(
                        plugin_dir,
                        state.get("Image").and_then(Value::as_str),
                        "state Image",
                        &mut errors,
                    );
                }
            }
        }
    }

    Ok(errors)
}

pub(crate) fn is_valid_plugin_version(version: &str) -> bool {
    let parts: Vec<&str> = version.split('.').collect();
    (1..=4).contains(&parts.len()) && parts.iter().all(|p| is_version_component(p))
}

fn is_version_component(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    if s == "0" {
        return true;
    }
    !s.starts_with('0') && s.chars().all(|c| c.is_ascii_digit()) && s.parse::<u32>().is_ok()
}

fn check_code_paths(
    plugin_dir: &Path,
    manifest: &Value,
    require_all_os: bool,
    errors: &mut Vec<String>,
) {
    if require_all_os
        && let Some(os_list) = manifest.get("OS").and_then(Value::as_array)
        && !os_list.is_empty()
    {
        for os in os_list {
            match os.get("Platform").and_then(Value::as_str) {
                Some("mac") => require_code_path(
                    plugin_dir,
                    manifest,
                    &["CodePathMac", "CodePath"],
                    "mac",
                    errors,
                ),
                Some("windows") => require_code_path(
                    plugin_dir,
                    manifest,
                    &["CodePathWin", "CodePath"],
                    "windows",
                    errors,
                ),
                Some(other) => errors.push(format!("unsupported OS.Platform `{other}`")),
                None => errors.push("OS entry is missing Platform".into()),
            }
        }
        return;
    }

    match current_platform_code_path(manifest) {
        Ok(Some(rel)) => match existing_file_in_plugin(plugin_dir, rel) {
            Ok(true) => {}
            Ok(false) => errors.push(format!("missing plugin binary: {rel}")),
            Err(()) => errors.push(format!(
                "plugin binary path is outside the plugin directory: {rel}"
            )),
        },
        Ok(None) => {
            errors.push("manifest.json is missing CodePath / CodePathMac / CodePathWin".into())
        }
        Err(err) => errors.push(err),
    }
}

fn require_code_path(
    plugin_dir: &Path,
    manifest: &Value,
    keys: &[&str],
    platform: &str,
    errors: &mut Vec<String>,
) {
    let Some(rel) = keys
        .iter()
        .find_map(|key| manifest.get(*key).and_then(Value::as_str))
    else {
        errors.push(format!(
            "manifest.json is missing {} for OS.Platform `{platform}`",
            keys.join(" / ")
        ));
        return;
    };
    match existing_file_in_plugin(plugin_dir, rel) {
        Ok(true) => {}
        Ok(false) => errors.push(format!("missing plugin binary for {platform}: {rel}")),
        Err(()) => errors.push(format!(
            "plugin binary path is outside the plugin directory: {rel}"
        )),
    }
}

fn current_platform_code_path(manifest: &Value) -> Result<Option<&str>, String> {
    let specific = if cfg!(windows) {
        manifest.get("CodePathWin").and_then(Value::as_str)
    } else if cfg!(target_os = "macos") {
        manifest.get("CodePathMac").and_then(Value::as_str)
    } else {
        return Err("Stream Deck plugins can only be validated on macOS or Windows".into());
    };
    Ok(specific.or_else(|| manifest.get("CodePath").and_then(Value::as_str)))
}

fn check_icon(plugin_dir: &Path, manifest: &Value, key: &str, errors: &mut Vec<String>) {
    check_icon_value(
        plugin_dir,
        manifest.get(key).and_then(Value::as_str),
        key,
        errors,
    );
}

fn check_icon_value(plugin_dir: &Path, rel: Option<&str>, label: &str, errors: &mut Vec<String>) {
    let Some(rel) = rel else {
        return;
    };
    match icon_exists(plugin_dir, rel) {
        Ok(true) => {}
        Ok(false) => errors.push(format!("missing {label}: {rel}")),
        Err(()) => errors.push(format!(
            "{label} path is outside the plugin directory: {rel}"
        )),
    }
}

fn icon_exists(plugin_dir: &Path, rel: &str) -> Result<bool, ()> {
    if !is_safe_relative(rel) {
        return Err(());
    }
    let candidates = [
        plugin_dir.join(rel),
        plugin_dir.join(rel).with_extension("png"),
        plugin_dir.join(rel).with_extension("svg"),
        plugin_dir.join(format!("{rel}.png")),
        plugin_dir.join(format!("{rel}.svg")),
        plugin_dir.join(format!("{rel}@2x.png")),
    ];
    Ok(candidates
        .iter()
        .any(|path| contained_file(plugin_dir, path)))
}

fn existing_file_in_plugin(plugin_dir: &Path, rel: &str) -> Result<bool, ()> {
    if !is_safe_relative(rel) {
        return Err(());
    }
    Ok(contained_file(plugin_dir, &plugin_dir.join(rel)))
}

fn is_safe_relative(rel: &str) -> bool {
    let path = Path::new(rel);
    !rel.is_empty()
        && !path.is_absolute()
        && path.components().all(|c| matches!(c, Component::Normal(_)))
}

fn contained_file(plugin_dir: &Path, joined: &Path) -> bool {
    let Ok(root) = plugin_dir.canonicalize() else {
        return joined.is_file();
    };
    joined
        .canonicalize()
        .is_ok_and(|path| path.starts_with(&root) && path.is_file())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn write_plugin(plugin: &Path, manifest: &str) {
        std::fs::create_dir_all(plugin).unwrap();
        std::fs::write(plugin.join("manifest.json"), manifest).unwrap();
    }

    #[test]
    fn reports_missing_manifest() {
        let dir = tempdir().unwrap();
        let plugin = dir.path().join("com.example.test.sdPlugin");
        std::fs::create_dir_all(&plugin).unwrap();
        let errors = validate_plugin(&plugin).unwrap();
        assert!(
            errors
                .iter()
                .any(|e| e.contains("manifest.json is missing"))
        );
    }

    #[test]
    fn rejects_escaping_manifest_paths() {
        let dir = tempdir().unwrap();
        let plugin = dir.path().join("com.example.escape.sdPlugin");
        write_plugin(
            &plugin,
            r#"{
                "UUID": "com.example.escape",
                "Version": "0.1.0.0",
                "Icon": "../secret",
                "CodePathMac": "/tmp/outside",
                "CodePathWin": "C:\\Windows\\outside.exe"
            }"#,
        );
        let errors = validate_plugin(&plugin).unwrap();
        assert!(
            errors
                .iter()
                .any(|e| e.contains("outside the plugin directory"))
        );
    }

    #[test]
    fn rejects_invalid_versions() {
        assert!(is_valid_plugin_version("1.2"));
        assert!(is_valid_plugin_version("1.2.3.4"));
        assert!(!is_valid_plugin_version("1.foo"));
        assert!(!is_valid_plugin_version("1.2.3.4.5"));
        assert!(!is_valid_plugin_version("4294967296"));
    }

    #[test]
    fn pack_validation_requires_declared_os_binaries() {
        let dir = tempdir().unwrap();
        let plugin = dir.path().join("com.example.os.sdPlugin");
        write_plugin(
            &plugin,
            r#"{
                "UUID": "com.example.os",
                "Version": "0.1.0.0",
                "CodePathMac": "bin/os",
                "CodePathWin": "bin/os.exe",
                "OS": [
                    { "Platform": "mac", "MinimumVersion": "12" },
                    { "Platform": "windows", "MinimumVersion": "10" }
                ]
            }"#,
        );
        std::fs::create_dir_all(plugin.join("bin")).unwrap();
        std::fs::write(plugin.join("bin/os"), b"mac").unwrap();
        let errors = validate_plugin_for_pack(&plugin).unwrap();
        assert!(
            errors
                .iter()
                .any(|e| e.contains("missing plugin binary for windows"))
        );
    }

    #[test]
    fn accepts_svg_icons() {
        let dir = tempdir().unwrap();
        let plugin = dir.path().join("com.example.svg.sdPlugin");
        write_plugin(
            &plugin,
            r#"{
                "UUID": "com.example.svg",
                "Version": "0.1.0.0",
                "Icon": "imgs/plugin/marketplace",
                "CodePathMac": "bin/plugin",
                "CodePathWin": "bin/plugin.exe",
                "Actions": [{
                    "UUID": "com.example.svg.one",
                    "Icon": "imgs/actions/icon",
                    "States": [{ "Image": "imgs/actions/key" }]
                }]
            }"#,
        );
        std::fs::create_dir_all(plugin.join("imgs/plugin")).unwrap();
        std::fs::create_dir_all(plugin.join("imgs/actions")).unwrap();
        std::fs::create_dir_all(plugin.join("bin")).unwrap();
        std::fs::write(plugin.join("imgs/plugin/marketplace.svg"), b"<svg/>").unwrap();
        std::fs::write(plugin.join("imgs/actions/icon.svg"), b"<svg/>").unwrap();
        std::fs::write(plugin.join("imgs/actions/key.svg"), b"<svg/>").unwrap();
        let errors = validate_plugin(&plugin).unwrap();
        assert!(
            !errors.iter().any(|e| e.contains("missing Icon")
                || e.contains("missing action Icon")
                || e.contains("missing state Image")),
            "expected SVG icons to validate, got {errors:?}"
        );
    }
}
