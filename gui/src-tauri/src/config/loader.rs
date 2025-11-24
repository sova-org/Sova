use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;
use super::types::Config;
use super::validation::Validate;

pub struct ConfigLoader {
    config_path: PathBuf,
}

impl ConfigLoader {
    pub fn new() -> Result<Self> {
        let config_dir = dirs::config_dir()
            .context("Failed to determine config directory")?
            .join("sova");

        fs::create_dir_all(&config_dir)
            .context("Failed to create config directory")?;

        let config_path = config_dir.join("config.toml");

        Ok(Self { config_path })
    }

    pub fn config_path(&self) -> &PathBuf {
        &self.config_path
    }

    pub fn load_or_create(&self) -> Result<Config> {
        if !self.config_path.exists() {
            let default_config = Config::default();
            self.save(&default_config)?;
            Ok(default_config)
        } else {
            self.load_and_normalize()
        }
    }

    pub fn load(&self) -> Result<Config> {
        let content = fs::read_to_string(&self.config_path)
            .context("Failed to read config file")?;

        let mut config: Config = toml::from_str(&content)
            .unwrap_or_else(|e| {
                sova_core::log_error!("Failed to parse config: {}. Using defaults.", e);
                Config::default()
            });

        config.validate();
        Ok(config)
    }

    fn load_and_normalize(&self) -> Result<Config> {
        let content = fs::read_to_string(&self.config_path)
            .context("Failed to read config file")?;

        let mut config: Config = match toml::from_str(&content) {
            Ok(c) => c,
            Err(e) => {
                let backup_path = self.config_path.with_extension("toml.backup");
                fs::write(&backup_path, &content)
                    .context("Failed to write backup")?;

                sova_core::log_error!(
                    "Config file corrupted: {}. Backup saved to {:?}. Using defaults.",
                    e, backup_path
                );

                let default = Config::default();
                self.save(&default)?;
                return Ok(default);
            }
        };

        config.validate();

        let current_toml = toml::to_string_pretty(&config)
            .context("Failed to serialize config")?;

        if content.trim() != current_toml.trim() {
            self.save(&config)?;
        }

        Ok(config)
    }

    pub fn save(&self, config: &Config) -> Result<()> {
        let toml_string = toml::to_string_pretty(config)
            .context("Failed to serialize config")?;

        fs::write(&self.config_path, toml_string)
            .context("Failed to write config file")?;

        Ok(())
    }
}
