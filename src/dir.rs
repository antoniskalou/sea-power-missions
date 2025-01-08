use std::{
    fs,
    path::{Path, PathBuf},
};

const MISSION_DIR: &str = r"Sea Power_Data\StreamingAssets\user\missions";
const ORIGINAL_DIR: &str = r"Sea Power_Data\StreamingAssets\original";
const AIRCRAFT_DIR: &str = "aircraft";
const VESSEL_DIR: &str = "vessels";

pub fn config_dir() -> PathBuf {
    // very unlikely to fail (unless you're running something older than
    // windows vista)
    //
    // as far as I understand it, this will not fail on windows unless either
    // the "folder ID" doesn't exist or the referenced "folder" doesn't have a
    // path (e.g. printer)
    //
    // see https://learn.microsoft.com/en-us/windows/win32/api/shlobj_core/nf-shlobj_core-shgetknownfolderpath#return-value
    dirs::config_dir()
        .expect("failed to find config path")
        .join("spmg")
}

pub fn find_root_dir() -> Option<PathBuf> {
    check_known_locations()
}

pub fn mission_dir(root_dir: &Path) -> PathBuf {
    root_dir.join(MISSION_DIR)
}

pub fn original_dir(root_dir: &Path) -> PathBuf {
    root_dir.join(ORIGINAL_DIR)
}

pub fn aircraft_dir(root_dir: &Path) -> PathBuf {
    original_dir(root_dir).join(AIRCRAFT_DIR)
}

pub fn vessel_dir(root_dir: &Path) -> PathBuf {
    original_dir(root_dir).join(VESSEL_DIR)
}

fn check_known_locations() -> Option<PathBuf> {
    // default install locations
    let default_paths = vec![
        r"C:\Program Files\Sea Power",
        r"C:\Program Files (x86)\Sea Power",
        r"C:\Games\Sea Power",
        r"D:\Games\Sea Power",
    ];

    eprintln!("checking known locations...");
    for path in default_paths {
        eprintln!("\t{}", path);
        if Path::new(path).exists() {
            return Some(path.into());
        }
    }
    eprintln!("\tFAILED");

    // maybe its in the steam folder?
    eprintln!("checking steam path...");
    if let Some(steam_path) = detect_steam_game_folder("Sea Power") {
        return Some(steam_path);
    }
    eprintln!("\tFAILED");

    // what about the registry?
    #[cfg(target_os = "windows")]
    {
        eprintln!("checking windows registry...");
        if let Some(registry_path) = check_registry_key("Sea Power") {
            return Some(registry_path);
        }
        eprintln!("\tFAILED");
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
        .filter(|location| location.contains(game_name))
        .map(PathBuf::from)
        .find(|path| path.exists())
}
