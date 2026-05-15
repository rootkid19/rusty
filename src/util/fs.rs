use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

pub fn expand_tilde(path: &str) -> Result<PathBuf> {
    if path == "~" {
        return home_dir();
    }

    if let Some(rest) = path.strip_prefix("~/") {
        return Ok(home_dir()?.join(rest));
    }

    Ok(PathBuf::from(path))
}

pub fn normalize_path(path: PathBuf) -> Result<PathBuf> {
    if path.starts_with("~") {
        return expand_tilde(
            path.to_str()
                .ok_or_else(|| anyhow::anyhow!("path contains invalid UTF-8"))?,
        );
    }
    Ok(path)
}

pub fn resolve_config_path(
    configured: Option<&str>,
    default_path: &str,
    root_dir: &Path,
) -> Result<PathBuf> {
    match configured {
        Some(path) => {
            let expanded = expand_tilde(path)?;
            if expanded.is_absolute() {
                Ok(expanded)
            } else {
                Ok(root_dir.join(expanded))
            }
        }
        None => {
            if is_default_root(root_dir)? {
                return expand_tilde(default_path);
            }

            let default_name = Path::new(default_path)
                .file_name()
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from(default_path));
            Ok(root_dir.join(default_name))
        }
    }
}

pub fn ensure_dir(path: &Path) -> Result<()> {
    fs::create_dir_all(path).with_context(|| format!("failed to create {}", path.display()))?;
    Ok(())
}

fn home_dir() -> Result<PathBuf> {
    env::var_os("HOME")
        .map(PathBuf::from)
        .context("HOME is not set")
}

fn is_default_root(root_dir: &Path) -> Result<bool> {
    Ok(root_dir == home_dir()?.join(".rusty"))
}
