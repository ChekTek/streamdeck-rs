use std::fs::File;
use std::io::copy;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use colored::Colorize;
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use serde_json::Value;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

use crate::project::resolve_project;
use crate::stream_deck::plugin_id_from_path;

use super::build::build_if_cargo;
use super::validate::validate_plugin;

const DEFAULT_IGNORES: &[&str] = &[".sdignore", ".git", "/.env*", "*.log", "logs"];

pub fn run(
    path: Option<PathBuf>,
    dry_run: bool,
    force: bool,
    output: Option<PathBuf>,
    version: Option<String>,
    ignore_validation: bool,
) -> Result<()> {
    let input = path.clone();
    let project = resolve_project(path)?;
    let plugin_dir = project.plugin_dir.clone();
    let uuid = plugin_id_from_path(&plugin_dir).context("invalid plugin directory")?;

    build_if_cargo(input)?;

    let version_guard = ManifestVersion::apply(&plugin_dir, version.as_deref())?;

    let errors = validate_plugin(&plugin_dir)?;
    if !errors.is_empty() {
        for error in &errors {
            println!("{}", error.yellow());
        }
        if ignore_validation {
            println!("Ignore validation flag found, bypassing validation errors");
        } else {
            version_guard.restore();
            bail!("validation failed with {} error(s)", errors.len());
        }
    }

    let output_dir = output.unwrap_or_else(|| std::env::current_dir().expect("cwd"));
    std::fs::create_dir_all(&output_dir)?;
    let output_path = output_dir.join(format!("{uuid}.streamDeckPlugin"));
    if output_path.exists() {
        if force {
            std::fs::remove_file(&output_path)?;
        } else {
            version_guard.restore();
            bail!(
                "File already exists\nSpecify a different -o|--output location, or -f|--force to overwrite"
            );
        }
    }

    let files = collect_files(&plugin_dir)?;
    let manifest: Value =
        serde_json::from_str(&std::fs::read_to_string(plugin_dir.join("manifest.json"))?)?;
    let name = manifest
        .get("Name")
        .and_then(Value::as_str)
        .unwrap_or(&uuid);
    let ver = manifest
        .get("Version")
        .and_then(Value::as_str)
        .unwrap_or("0.0.0.0");

    let unpacked: u64 = files.iter().map(|f| f.size).sum();
    let size_pad = files
        .iter()
        .map(|f| size_as_string(f.size).len())
        .max()
        .unwrap_or(0);

    println!(" {name} (v{ver})");
    println!();
    println!("{}", "Plugin Contents".cyan());
    for (i, file) in files.iter().enumerate() {
        let branch = if i + 1 == files.len() {
            "└─"
        } else {
            "├─"
        };
        println!(
            "{}  {:<width$}  {}",
            branch.dimmed(),
            size_as_string(file.size),
            file.relative,
            width = size_pad
        );
    }
    println!();
    println!("{}", "Plugin Details".cyan());
    println!("  Name:           {name}");
    println!("  Version:        {ver}");
    println!("  UUID:           {uuid}");
    println!("  Total files:    {}", files.len());
    println!("  Unpacked size:  {}", size_as_string(unpacked));
    println!(
        "  File name:      {}",
        output_path.file_name().unwrap().to_string_lossy()
    );
    println!();

    if dry_run {
        version_guard.restore();
        println!("No package created, --dry-run flag is present");
        println!("{}", output_path.display().to_string().dimmed());
        return Ok(());
    }

    write_zip(&plugin_dir, &files, &output_path)?;
    version_guard.restore();
    println!("{}", "Successfully packaged plugin".green());
    println!("{}", output_path.display());
    Ok(())
}

struct PackedFile {
    relative: String,
    absolute: PathBuf,
    size: u64,
}

fn collect_files(plugin_dir: &Path) -> Result<Vec<PackedFile>> {
    let gi = build_ignore(plugin_dir)?;
    let mut files = Vec::new();
    for entry in walkdir_files(plugin_dir)? {
        let rel = entry
            .strip_prefix(plugin_dir)?
            .to_string_lossy()
            .replace('\\', "/");
        if gi.matched(&rel, false).is_ignore() {
            continue;
        }
        let size = std::fs::metadata(&entry)?.len();
        files.push(PackedFile {
            relative: rel,
            absolute: entry,
            size,
        });
    }
    files.sort_by(|a, b| a.relative.cmp(&b.relative));
    Ok(files)
}

fn build_ignore(plugin_dir: &Path) -> Result<Gitignore> {
    let mut builder = GitignoreBuilder::new(plugin_dir);
    for pattern in DEFAULT_IGNORES {
        builder.add_line(None, pattern)?;
    }
    let sdignore = plugin_dir.join(".sdignore");
    if sdignore.is_file()
        && let Some(err) = builder.add(&sdignore)
    {
        return Err(err.into());
    }
    Ok(builder.build()?)
}

fn walkdir_files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    fn rec(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                rec(&path, files)?;
            } else if path.is_file() {
                files.push(path);
            }
        }
        Ok(())
    }
    rec(root, &mut files)?;
    Ok(files)
}

fn write_zip(plugin_dir: &Path, files: &[PackedFile], output: &Path) -> Result<()> {
    let prefix = plugin_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("plugin.sdPlugin");
    let file = File::create(output)?;
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    for packed in files {
        let name = format!("{prefix}/{}", packed.relative);
        zip.start_file(name, options)?;
        let mut src = File::open(&packed.absolute)?;
        copy(&mut src, &mut zip)?;
    }
    zip.finish()?;
    Ok(())
}

struct ManifestVersion {
    path: PathBuf,
    original: Option<String>,
}

impl ManifestVersion {
    fn apply(plugin_dir: &Path, version: Option<&str>) -> Result<Self> {
        let path = plugin_dir.join("manifest.json");
        if !path.is_file() {
            return Ok(Self {
                path,
                original: None,
            });
        }
        let original = std::fs::read_to_string(&path)?;
        let mut manifest: Value = serde_json::from_str(&original)?;
        let current = manifest
            .get("Version")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let version = version.unwrap_or(&current);
        let padded = pad_version(version);
        if let Some(obj) = manifest.as_object_mut() {
            obj.insert("Version".into(), Value::String(padded));
        }
        std::fs::write(&path, serde_json::to_string_pretty(&manifest)?)?;
        Ok(Self {
            path,
            original: Some(original),
        })
    }

    fn restore(&self) {
        if let Some(original) = &self.original {
            let _ = std::fs::write(&self.path, original);
        }
    }
}

impl Drop for ManifestVersion {
    fn drop(&mut self) {
        self.restore();
    }
}

fn pad_version(version: &str) -> String {
    let mut parts: Vec<&str> = version.split('.').filter(|p| !p.is_empty()).collect();
    while parts.len() < 4 {
        parts.push("0");
    }
    parts.join(".")
}

fn size_as_string(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    if bytes < 1024 {
        format!("{bytes} B")
    } else if (bytes as f64) < MB {
        format!("{:.1} KB", bytes as f64 / KB)
    } else {
        format!("{:.1} MB", bytes as f64 / MB)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn pads_versions() {
        assert_eq!(pad_version("1.2"), "1.2.0.0");
        assert_eq!(pad_version("1.2.3.4"), "1.2.3.4");
    }

    #[test]
    fn packs_plugin_zip() {
        let dir = tempdir().unwrap();
        let plugin = dir.path().join("com.example.pack.sdPlugin");
        std::fs::create_dir_all(plugin.join("bin")).unwrap();
        std::fs::write(
            plugin.join("manifest.json"),
            r#"{
                "Name": "Pack",
                "UUID": "com.example.pack",
                "Version": "0.1.0.0",
                "CodePathMac": "bin/pack",
                "CodePathWin": "bin/pack.exe",
                "Icon": "imgs/icon",
                "CategoryIcon": "imgs/icon"
            }"#,
        )
        .unwrap();
        std::fs::create_dir_all(plugin.join("imgs")).unwrap();
        std::fs::write(plugin.join("imgs/icon.png"), b"png").unwrap();
        std::fs::write(plugin.join("bin/pack"), b"binary").unwrap();
        std::fs::write(plugin.join("bin/pack.exe"), b"binary").unwrap();
        std::fs::write(plugin.join(".sdignore"), "logs\n").unwrap();
        std::fs::create_dir_all(plugin.join("logs")).unwrap();
        std::fs::write(plugin.join("logs/x.log"), "nope").unwrap();

        let out = dir.path().join("out");
        std::fs::create_dir_all(&out).unwrap();
        run(
            Some(plugin.clone()),
            false,
            true,
            Some(out.clone()),
            None,
            true,
        )
        .unwrap();
        let zip_path = out.join("com.example.pack.streamDeckPlugin");
        assert!(zip_path.is_file());
        let bytes = std::fs::read(&zip_path).unwrap();
        assert!(bytes.len() > 20);
    }
}
