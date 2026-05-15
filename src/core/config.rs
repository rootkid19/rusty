use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::core::metadata::NoteStatus;
use crate::util::fs as util_fs;

const DEFAULT_ROOT_DIR: &str = "~/.rusty";
const DEFAULT_NOTES_DIR: &str = "~/.rusty/notes";
const DEFAULT_DAILY_DIR: &str = "~/.rusty/daily";
const DEFAULT_TEMPLATES_DIR: &str = "~/.rusty/templates";
const DEFAULT_CACHE_DIR: &str = "~/.rusty/cache";
const DEFAULT_CONFIG_PATH: &str = "~/.rusty/config.toml";
const DEFAULT_EDITOR: &str = "nvim";
const DEFAULT_PREVIEW_LINES: usize = 20;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub root_dir: PathBuf,
    pub notes_dir: PathBuf,
    pub daily_dir: PathBuf,
    pub templates_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub config_path: PathBuf,
    pub default_editor: String,
    pub default_status: NoteStatus,
    pub preview_lines: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct ConfigFile {
    notes_dir: Option<String>,
    daily_dir: Option<String>,
    templates_dir: Option<String>,
    cache_dir: Option<String>,
    default_editor: Option<String>,
    default_status: Option<NoteStatus>,
    preview_lines: Option<usize>,
}

impl Config {
    pub fn load() -> Result<Self> {
        let root_dir = util_fs::expand_tilde(DEFAULT_ROOT_DIR)?;
        let config_path = util_fs::expand_tilde(DEFAULT_CONFIG_PATH)?;
        Self::load_from_paths(root_dir, config_path)
    }

    pub fn load_from_paths(root_dir: PathBuf, config_path: PathBuf) -> Result<Self> {
        let root_dir = util_fs::normalize_path(root_dir)?;
        let config_path = util_fs::normalize_path(config_path)?;
        let raw = if config_path.exists() {
            let contents = fs::read_to_string(&config_path)
                .with_context(|| format!("failed to read config at {}", config_path.display()))?;
            toml::from_str::<ConfigFile>(&contents)
                .with_context(|| format!("malformed config at {}", config_path.display()))?
        } else {
            ConfigFile::default()
        };

        Ok(Self {
            root_dir: root_dir.clone(),
            notes_dir: util_fs::resolve_config_path(
                raw.notes_dir.as_deref(),
                DEFAULT_NOTES_DIR,
                &root_dir,
            )?,
            daily_dir: util_fs::resolve_config_path(
                raw.daily_dir.as_deref(),
                DEFAULT_DAILY_DIR,
                &root_dir,
            )?,
            templates_dir: util_fs::resolve_config_path(
                raw.templates_dir.as_deref(),
                DEFAULT_TEMPLATES_DIR,
                &root_dir,
            )?,
            cache_dir: util_fs::resolve_config_path(
                raw.cache_dir.as_deref(),
                DEFAULT_CACHE_DIR,
                &root_dir,
            )?,
            config_path,
            default_editor: raw
                .default_editor
                .unwrap_or_else(|| DEFAULT_EDITOR.to_string()),
            default_status: raw.default_status.unwrap_or(NoteStatus::Active),
            preview_lines: raw.preview_lines.unwrap_or(DEFAULT_PREVIEW_LINES),
        })
    }

    pub fn for_root(root_dir: impl AsRef<Path>) -> Result<Self> {
        let root_dir = util_fs::normalize_path(root_dir.as_ref().to_path_buf())?;
        let config_path = root_dir.join("config.toml");
        Self::load_from_paths(root_dir, config_path)
    }

    pub fn ensure_layout(&self) -> Result<()> {
        util_fs::ensure_dir(&self.root_dir)?;
        util_fs::ensure_dir(&self.notes_dir)?;
        util_fs::ensure_dir(&self.daily_dir)?;
        util_fs::ensure_dir(&self.templates_dir)?;
        util_fs::ensure_dir(&self.cache_dir)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use tempfile::tempdir;

    use super::Config;
    use crate::core::metadata::NoteStatus;

    #[test]
    fn missing_config_uses_defaults() -> Result<()> {
        let temp = tempdir()?;
        let config = Config::for_root(temp.path())?;

        assert_eq!(config.root_dir, temp.path());
        assert_eq!(config.notes_dir, temp.path().join("notes"));
        assert_eq!(config.daily_dir, temp.path().join("daily"));
        assert_eq!(config.default_status, NoteStatus::Active);
        assert_eq!(config.preview_lines, 20);
        Ok(())
    }
}
