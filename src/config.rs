use crate::error::{NoterError, Result};
use dirs::home_dir;
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub notes_dir: PathBuf,
    pub db_path: PathBuf,
    pub default_extension: String,
    pub editor: Option<String>,
    pub encryption_key: String,
    pub export_dir: PathBuf,
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_dir = Self::config_dir()?;
        let config_path = config_dir.join("config.toml");

        if !config_path.exists() {
            let config = Config::default();
            config.save()?;
            return Ok(config);
        }

        let contents = std::fs::read_to_string(config_path)?;
        toml::from_str(&contents).map_err(|e| NoterError::Config(e.to_string()))
    }

    pub fn save(&self) -> Result<()> {
        let config_dir = Self::config_dir()?;
        let config_path = config_dir.join("config.toml");

        fs::create_dir_all(&config_dir)?;
        fs::create_dir_all(&self.notes_dir)?;
        fs::create_dir_all(self.db_path.parent().unwrap())?;

        let toml = toml::to_string_pretty(self).map_err(|e| NoterError::Config(e.to_string()))?;
        fs::write(config_path, toml)?;

        Ok(())
    }

    fn config_dir() -> Result<PathBuf> {
        home_dir()
            .map(|home| home.join(".config").join("noters"))
            .ok_or(NoterError::HomeDirNotFound)
    }

    fn generate_encryption_key() -> String {
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect()
    }
}

impl Default for Config {
    fn default() -> Self {
        let home = home_dir().unwrap_or_default();
        let noters_dir = home.join(".noters");
        Self {
            notes_dir: noters_dir.join("notes"),
            db_path: noters_dir.join("noters.db"),
            default_extension: String::from("md"),
            editor: None,
            encryption_key: Self::generate_encryption_key(),
            export_dir: noters_dir.join("exports"),
        }
    }
}
