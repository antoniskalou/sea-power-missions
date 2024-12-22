use std::path::PathBuf;

const ROOT_DIR: &str = "D:\\SteamLibrary\\steamapps\\common\\Sea Power";
const MISSION_DIR: &str = "Sea Power_Data\\StreamingAssets\\user\\missions";
const ORIGINAL_DIR: &str = "Sea Power_Data\\StreamingAssets\\original";
const AIRCRAFT_DIR: &str = "aircraft";
const VESSEL_DIR: &str = "vessels";

pub fn root_dir() -> PathBuf {
    PathBuf::from(ROOT_DIR)
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
