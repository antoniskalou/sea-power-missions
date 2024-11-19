use std::path::PathBuf;

const ROOT_DIR: &'static str = "D:\\SteamLibrary\\steamapps\\common\\Sea Power";
const MISSION_DIR: &'static str = "Sea Power_Data\\StreamingAssets\\user\\missions";
const AIRCRAFT_DIR: &'static str = "Sea Power_Data\\StreamingAssets\\original\\aircraft";
const VESSEL_DIR: &'static str = "Sea Power_Data\\StreamingAssets\\original\\vessels";

pub fn root_dir() -> PathBuf {
    PathBuf::from(ROOT_DIR)
}

pub fn mission_dir() -> PathBuf {
    root_dir().join(MISSION_DIR)
}

pub fn aircraft_dir() -> PathBuf {
    root_dir().join(AIRCRAFT_DIR)
}

pub fn vessel_dir() -> PathBuf {
    root_dir().join(VESSEL_DIR)
}
