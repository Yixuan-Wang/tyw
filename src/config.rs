use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Deserialize, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub py: PyConfig,
    #[serde(default)]
    pub tg: TgConfig,
}

#[derive(Deserialize, Default)]
pub struct PyConfig {
    #[serde(default)]
    pub env: PyEnvConfig,
}

#[derive(Deserialize, Default)]
pub struct PyEnvConfig {
    #[serde(default)]
    pub home: String,
}

#[derive(Deserialize, Default)]
pub struct TgConfig {
    #[serde(default)]
    pub token: String,
    #[serde(default)]
    pub chat_id: String,
}

impl AppConfig {
    pub fn load(config_path: Option<&str>) -> Result<Self> {
        let path = if let Some(p) = config_path {
            PathBuf::from(p)
        } else {
            Self::find_config_file()
        };

        let contents = match std::fs::read_to_string(&path) {
            Ok(c) => {
                log::info!("Loaded config file: {}", path.display());
                c
            }
            Err(_) => {
                log::error!("Failed to load config from {}", path.display());
                return Ok(AppConfig::default());
            }
        };

        let mut config: AppConfig =
            toml::from_str(&contents).context("Failed to parse config file")?;

        // Override tg.token from TELEGRAM_TOKEN env var if set
        if let Ok(token) = std::env::var("TELEGRAM_TOKEN") {
            config.tg.token = token;
        }

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
