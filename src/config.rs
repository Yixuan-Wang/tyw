use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::py::PyConfig;
use crate::tg::TgConfig;

#[derive(Deserialize, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub py: PyConfig,
    #[serde(default)]
    pub tg: TgConfig,
}

impl AppConfig {
    pub fn load(config_path: Option<&Path>) -> Result<Self> {
        let has_designated_config = config_path.is_some();
        let path = match config_path {
            Some(p) => p.to_path_buf(),
            None => Self::find_config_file(),
        };

        let mut config = match std::fs::read_to_string(&path) {
            Ok(contents) => {
                log::info!("Loaded config file: {}", path.display());
                toml::from_str(&contents).context("Failed to parse config file")?
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound && !has_designated_config => {
                AppConfig::default()
            }
            Err(e) => {
                return Err(e)
                    .with_context(|| format!("Failed to read config from {}", path.display()));
            }
        };

        config.tg.apply_env_overrides();

        Ok(config)
    }

    fn find_config_file() -> PathBuf {
        // Try $XDG_CONFIG_HOME first (dirs::config_dir handles this)
        if let Some(config_dir) = dirs::config_dir() {
            let p = config_dir.join("tyw.toml");
            if p.exists() {
                return p;
            }
        }

        // Fallback to ~/.config/tyw.toml
        if let Some(home) = dirs::home_dir() {
            let p = home.join(".config").join("tyw.toml");
            if p.exists() {
                return p;
            }
        }

        // Return default path even if it doesn't exist
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("tyw.toml")
    }
}
