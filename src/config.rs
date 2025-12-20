use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;

/// Configuration for csep
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    /// Default model to use
    pub default_model: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_model: "all-minilm-l6-v2".to_string(),
        }
    }
}

/// Get the configuration file path
pub fn get_config_path() -> PathBuf {
    let config_dir = match env::var("XDG_CONFIG_HOME") {
        Ok(path) => PathBuf::from(path),
        Err(_) => {
            let home = env::var("HOME").unwrap_or(".".to_string());
            PathBuf::from(home).join(".config")
        }
    };
    config_dir.join("csep").join("config.json")
}

/// Load configuration from file, or create default if not exists
pub fn load_config() -> Result<Config> {
    let config_path = get_config_path();

    // Create config directory if it doesn't exist
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Try to read config file
    if config_path.exists() {
        let config_content = fs::read_to_string(config_path)?;
        let config: Config = serde_json::from_str(&config_content)?;
        Ok(config)
    } else {
        // Return default config
        Ok(Config::default())
    }
}

/// Save configuration to file
pub fn save_config(config: &Config) -> Result<()> {
    let config_path = get_config_path();

    // Create config directory if it doesn't exist
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let config_json = serde_json::to_string_pretty(config)?;
    fs::write(config_path, config_json)?;
    Ok(())
}

/// Reset configuration to defaults
pub fn reset_config() -> Result<()> {
    let config = Config::default();
    save_config(&config)?;
    Ok(())
}
