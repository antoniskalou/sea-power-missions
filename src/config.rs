use configparser::ini;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Clone, Debug)]
pub enum ConfigLoadError {
    #[error("failed to parse ini file {path}: {reason}")]
    IniParse { path: PathBuf, reason: String },
    #[error("required key `{section}.{key}` is missing from config")]
    MissingKey { section: String, key: String },
}

#[derive(Clone, Debug)]
pub struct Config {
    pub game_path: PathBuf,
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, ConfigLoadError> {
        let config = load_config(path.as_ref())?;
        let game_path = fetch_key(&config, "general", "game_path")?.into();
        Ok(Config { game_path })
    }
}

fn fetch_key(config: &ini::Ini, section: &str, key: &str) -> Result<String, ConfigLoadError> {
    config.get(section, key).ok_or(ConfigLoadError::MissingKey {
        section: section.to_owned(),
        key: key.to_owned(),
    })
}

fn load_config(path: &Path) -> Result<ini::Ini, ConfigLoadError> {
    let mut config = ini::Ini::new();
    let _ = config
        .load(path)
        .map_err(|reason| ConfigLoadError::IniParse {
            path: path.to_owned(),
            reason,
        })?;
    Ok(config)
}
