use configparser::ini::Ini;
use std::{
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};

const MISSION_DIR: &str = r"Sea Power_Data\StreamingAssets\user\missions";
const ORIGINAL_DIR: &str = r"Sea Power_Data\StreamingAssets\original";
const AIRCRAFT_DIR: &str = "aircraft";
const VESSEL_DIR: &str = "vessels";

pub fn root_dir() -> PathBuf {
    // TODO: first try and load from app config
    try_load_config().or_else(check_known_locations).unwrap()
}

pub fn mission_dir() -> PathBuf {
    root_dir().join(MISSION_DIR)
}

pub fn original_dir() -> PathBuf {
    root_dir().join(ORIGINAL_DIR)
}

pub fn aircraft_dir() -> PathBuf {
    original_dir().join(AIRCRAFT_DIR)
}

pub fn vessel_dir() -> PathBuf {
    original_dir().join(VESSEL_DIR)
}

fn try_load_config() -> Option<PathBuf> {
    let config_file = dirs::config_dir()?.join("spmg").join("config.ini");
    let mut config = Ini::new();
    config.load(config_file).ok()?;
    config
        .get("general", "game_path")
        .and_then(|s| PathBuf::from_str(&s).ok())
}

fn check_known_locations() -> Option<PathBuf> {
    // default install locations
    let default_paths = vec![
        r"C:\Program Files\Sea Power",
        r"C:\Program Files (x86)\Sea Power",
        r"D:\Games\Sea Power",
    ];

    for path in default_paths {
        if Path::new(path).exists() {
            return Some(path.into());
        }
    }

    // maybe its in the steam folder?
    if let Some(steam_path) = detect_steam_game_folder("Sea Power") {
        return Some(steam_path);
    }

    // what about the registry?
    #[cfg(target_os = "windows")]
    if let Some(registry_path) = check_registry_key("Sea Power") {
        return Some(registry_path);
    }

    // not found...
    None
}

fn detect_steam_game_folder(game_name: &str) -> Option<PathBuf> {
    let steam_default_path = r"C:\Program Files (x86)\Steam\steamapps\libraryfolders.vdf";
    fs::read_to_string(steam_default_path)
        .ok()?
        .lines()
        // very naive and silly way of "parsing" the file for contained paths
        .filter(|line| line.contains("path"))
        .filter_map(|line| line.split('"').nth(3))
        .map(|lib_path| {
            let mut path = PathBuf::from(lib_path);
            path.extend(["steamapps", "common", game_name]);
            path
        })
        .find(|game_path| game_path.exists())
}

#[cfg(target_os = "windows")]
fn check_registry_key(game_name: &str) -> Option<PathBuf> {
    use winreg::enums::*;
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_LOCAL_MACHINE);
    let uninstall_key = hkcu
        .open_subkey_with_flags(
            r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
            KEY_READ,
        )
        .ok()?;

    let install_locations = uninstall_key
        .enum_keys()
        .flatten()
        .filter_map(|name| uninstall_key.open_subkey(&name).ok())
        .filter_map(|key| key.get_value::<String, _>("InstallLocation").ok());

    install_locations
        .filter(|install_location| install_location.contains(game_name))
        .map(PathBuf::from)
        .find(|path| path.exists())
}
