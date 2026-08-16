use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use colored::Colorize;
use serde_json::Value;

use crate::project::resolve_project;
use crate::stream_deck::{is_valid_plugin_id, plugin_id_from_path};

pub fn run(path: Option<PathBuf>) -> Result<()> {
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

    check_icon(plugin_dir, &manifest, "Icon", &mut errors);
    check_icon(plugin_dir, &manifest, "CategoryIcon", &mut errors);

    let code_path = current_platform_code_path(&manifest);
    if let Some(rel) = code_path {
        if !plugin_dir.join(rel).is_file() {
            errors.push(format!("missing plugin binary: {rel}"));
        }
    } else {
        errors.push("manifest.json is missing CodePath / CodePathMac / CodePathWin".into());
    }

    if let Some(actions) = manifest.get("Actions").and_then(Value::as_array) {
        for action in actions {
            check_icon_value(
                plugin_dir,
                action.get("Icon").and_then(Value::as_str),
                "action Icon",
                &mut errors,
            );
            if let Some(pi) = action.get("PropertyInspectorPath").and_then(Value::as_str)
                && !plugin_dir.join(pi).is_file()
            {
                errors.push(format!("missing property inspector: {pi}"));
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

fn current_platform_code_path(manifest: &Value) -> Option<&str> {
    let specific = if cfg!(windows) {
        manifest.get("CodePathWin").and_then(Value::as_str)
    } else {
        manifest.get("CodePathMac").and_then(Value::as_str)
    };
    specific.or_else(|| manifest.get("CodePath").and_then(Value::as_str))
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
    if icon_exists(plugin_dir, rel) {
        return;
    }
    errors.push(format!("missing {label}: {rel}"));
}

fn icon_exists(plugin_dir: &Path, rel: &str) -> bool {
    let base = plugin_dir.join(rel);
    base.is_file()
        || base.with_extension("png").is_file()
        || plugin_dir.join(format!("{rel}.png")).is_file()
        || plugin_dir.join(format!("{rel}@2x.png")).is_file()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

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
}
