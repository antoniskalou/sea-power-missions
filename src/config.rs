use configparser::ini;
use std::io;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigLoadError {
    #[error("failed to parse ini file {path}: {reason}")]
    IniParse { path: PathBuf, reason: String },
    #[error("required key `{section}.{key}` is missing from config")]
    MissingKey { section: String, key: String },
}

#[derive(Error, Debug)]
#[error("failed to write config file {path}: {source}")]
pub struct ConfigWriteError {
    path: PathBuf,
    #[source]
    source: io::Error,
}

#[derive(Clone, Debug)]
pub struct Config {
    /// The path where Sea Power is located.
    pub game_root: PathBuf,
}

impl Config {
    pub fn new<P: AsRef<Path>>(game_root: P) -> Self {
        Config {
            game_root: game_root.as_ref().to_owned(),
        }
    }

    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, ConfigLoadError> {
        let config = load_config(path.as_ref())?;
        let game_root = fetch_key(&config, "general", "game_root")?.into();
        Ok(Config { game_root })
    }

    pub fn save<P: AsRef<Path>>(self, path: P) -> Result<(), ConfigWriteError> {
        ini::Ini::new().write(&path).map_err(|e| ConfigWriteError {
            path: path.as_ref().to_owned(),
            source: e,
        })
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
