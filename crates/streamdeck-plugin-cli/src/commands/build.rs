use std::path::PathBuf;

use anyhow::Result;
use colored::Colorize;

use crate::project::{cargo_build_release, copy_release_binary, resolve_project};

pub fn run(path: Option<PathBuf>) -> Result<()> {
    let project = resolve_project(path)?;
    let Some(manifest) = project.cargo_manifest else {
        anyhow::bail!(
            "no Cargo.toml found next to {}",
            project.plugin_dir.display()
        );
    };
    cargo_build_release(&manifest)?;
    let dest = copy_release_binary(&manifest, &project.plugin_dir)?;
    println!("Built {}", dest.display().to_string().green());
    Ok(())
}

pub fn build_if_cargo(path: Option<PathBuf>) -> Result<()> {
    let project = resolve_project(path)?;
    if let Some(manifest) = project.cargo_manifest {
        cargo_build_release(&manifest)?;
        copy_release_binary(&manifest, &project.plugin_dir)?;
    }
    Ok(())
}
