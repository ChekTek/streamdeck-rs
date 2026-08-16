use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};
use serde::Deserialize;

use crate::stream_deck::{
    PLUGIN_SUFFIX, is_sdplugin_dir, plugin_id_from_path, require_supported_host,
};

#[derive(Debug, Clone)]
pub struct PluginProject {
    pub plugin_dir: PathBuf,
    pub cargo_manifest: Option<PathBuf>,
}

pub fn resolve_project(path: Option<PathBuf>) -> Result<PluginProject> {
    let path = path.unwrap_or_else(|| std::env::current_dir().expect("cwd"));
    let path = path.canonicalize().unwrap_or(path);

    if is_sdplugin_dir(&path) {
        let cargo_manifest = path.parent().and_then(|parent| {
            let manifest = parent.join("Cargo.toml");
            manifest.is_file().then_some(manifest)
        });
        return Ok(PluginProject {
            plugin_dir: path,
            cargo_manifest,
        });
    }

    if path.join("Cargo.toml").is_file() {
        let plugin_dir = find_sdplugin_child(&path)?;
        return Ok(PluginProject {
            plugin_dir,
            cargo_manifest: Some(path.join("Cargo.toml")),
        });
    }

    if let Some(plugin_dir) = find_sdplugin_child_opt(&path) {
        let cargo_manifest = path.join("Cargo.toml");
        return Ok(PluginProject {
            plugin_dir,
            cargo_manifest: cargo_manifest.is_file().then_some(cargo_manifest),
        });
    }

    bail!("no .sdPlugin directory found at {}", path.display());
}

fn find_sdplugin_child(path: &Path) -> Result<PathBuf> {
    find_sdplugin_child_opt(path).with_context(|| {
        format!(
            "expected a folder ending with {PLUGIN_SUFFIX} inside {}",
            path.display()
        )
    })
}

fn find_sdplugin_child_opt(path: &Path) -> Option<PathBuf> {
    let mut matches = Vec::new();
    let entries = std::fs::read_dir(path).ok()?;
    for entry in entries.flatten() {
        let child = entry.path();
        if plugin_id_from_path(&child).is_some() && child.is_dir() {
            matches.push(child);
        }
    }
    if matches.len() == 1 {
        matches.pop()
    } else {
        None
    }
}

#[derive(Deserialize)]
struct CargoMetadata {
    target_directory: PathBuf,
    packages: Vec<CargoPackage>,
}

#[derive(Deserialize)]
struct CargoPackage {
    manifest_path: PathBuf,
    targets: Vec<CargoTarget>,
}

#[derive(Deserialize)]
struct CargoTarget {
    name: String,
    kind: Vec<String>,
}

pub fn cargo_bin_and_target(manifest: &Path) -> Result<(String, PathBuf)> {
    let output = Command::new("cargo")
        .args([
            "metadata",
            "--format-version",
            "1",
            "--no-deps",
            "--manifest-path",
        ])
        .arg(manifest)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .context("failed to run cargo metadata")?;
    if !output.status.success() {
        bail!(
            "cargo metadata failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let meta: CargoMetadata = serde_json::from_slice(&output.stdout)?;
    let manifest_canon = manifest
        .canonicalize()
        .unwrap_or_else(|_| manifest.to_path_buf());
    let package = meta
        .packages
        .iter()
        .find(|p| {
            p.manifest_path == manifest_canon
                || p.manifest_path == *manifest
                || p.manifest_path.as_os_str() == manifest.as_os_str()
        })
        .or_else(|| meta.packages.first())
        .context("no package found in cargo metadata")?;
    let bin = package
        .targets
        .iter()
        .find(|t| t.kind.iter().any(|k| k == "bin"))
        .context("plugin crate has no binary target")?;
    Ok((bin.name.clone(), meta.target_directory))
}

pub fn release_bin_path(target_directory: &Path, bin_name: &str) -> Result<PathBuf> {
    let mut path = target_directory.join("release").join(bin_name);
    if cfg!(windows) {
        path.set_extension("exe");
    } else if !cfg!(target_os = "macos") {
        bail!("Stream Deck plugins can only be built on macOS or Windows");
    }
    Ok(path)
}

pub fn plugin_bin_path(plugin_dir: &Path, bin_name: &str) -> Result<PathBuf> {
    let file = if cfg!(windows) {
        format!("{bin_name}.exe")
    } else if cfg!(target_os = "macos") {
        bin_name.to_string()
    } else {
        bail!("Stream Deck plugins can only be built on macOS or Windows");
    };
    Ok(plugin_dir.join("bin").join(file))
}

pub fn copy_release_binary(manifest: &Path, plugin_dir: &Path) -> Result<PathBuf> {
    let (bin_name, target_dir) = cargo_bin_and_target(manifest)?;
    let src = release_bin_path(&target_dir, &bin_name)?;
    if !src.is_file() {
        bail!("release binary not found at {}", src.display());
    }
    let dest = plugin_bin_path(plugin_dir, &bin_name)?;
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::copy(&src, &dest)
        .with_context(|| format!("failed to copy {} -> {}", src.display(), dest.display()))?;
    Ok(dest)
}

pub fn cargo_build_release(manifest: &Path) -> Result<()> {
    require_supported_host()?;
    let output = Command::new("cargo")
        .args(["build", "--release", "--manifest-path"])
        .arg(manifest)
        .output()
        .context("failed to run cargo build")?;
    if !output.status.success() {
        bail!(
            "cargo build --release failed:\n{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(())
}
