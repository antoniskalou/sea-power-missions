use std::collections::HashMap;

use crate::dir;
use configparser::ini::Ini;
use std::{fs, io, path::Path};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum VesselType {
    Ship,
    Submarine,
    // TODO: figure out if this is necessary,
    // also consider using Option<VesselType>
    Unknown,
}

impl From<String> for VesselType {
    fn from(value: String) -> Self {
        match value.to_lowercase().as_str() {
            "vessel" => Self::Ship,
            "submarine" => Self::Submarine,
            _ => Self::Unknown,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Vessel {
    pub id: String,
    pub nation: String,
    pub subtype: VesselType,
}

#[derive(Clone, Debug)]
pub enum AircraftType {
    Helicopter,
    FixedWing,
    Unknown,
}

impl From<String> for AircraftType {
    fn from(value: String) -> Self {
        match value.to_lowercase().as_str() {
            "helicopter" => Self::Helicopter,
            "aircraft" => Self::FixedWing,
            _ => Self::Unknown,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Aircraft {
    pub id: String,
    pub nation: String,
    pub subtype: AircraftType,
}

/// Sea Power encodes unit information in the filename, usually structured
/// like <nation>_<vessel_name>
fn path_to_id(path: &Path) -> Option<&str> {
    path.file_stem().and_then(|p| p.to_str())
}

/// load a unit ini file
fn load_ini(path: &Path) -> Result<Ini, String> {
    let mut config = Ini::new();
    config.load(path)?;
    Ok(config)
}

fn load_nation_reference() -> Result<HashMap<String, String>, UnitDBError> {
    let config = load_ini(&dir::original_dir().join("nations_reference.ini"))?;

    let mut nations = HashMap::new();
    if let Some(map) = config.get_map() {
        for (_, nation) in map {
            let prefix = nation.get("nationprefix")
                .map(|o| (*o).clone())
                .flatten();
            let name = nation.get("nationname")
                .map(|o| (*o).clone())
                .flatten();
            prefix
                .zip(name)
                .map(|(prefix, name)| {
                    nations.insert(prefix, name);
                });
        }
    }
    Ok(nations)
}

fn load_vessels() -> Result<HashMap<String, Vessel>, UnitDBError> {
    let mut vessels = HashMap::new();
    for entry in fs::read_dir(dir::vessel_dir())? {
        let entry = entry?;
        let path = entry.path();
        let id = path_to_id(&path).unwrap(); // FIXME: unwrap

        // skip storing variants for now, TODO
        if id.ends_with("_variants") {
            continue;
        }

        if let Some((nation, _)) = id.split_once("_") {
            let id = id.to_owned();
            let nation = nation.to_owned();
            let config = load_ini(&path)?;
            let subtype = config
                .get("General", "UnitType")
                .map(|t| VesselType::from(t))
                .unwrap_or(VesselType::Unknown);
            vessels.insert(
                id.clone(),
                Vessel {
                    id,
                    nation,
                    subtype,
                },
            );
        }
    }
    Ok(vessels)
}

fn load_aircraft() -> Result<HashMap<String, Aircraft>, UnitDBError> {
    Ok(HashMap::new())
}

#[derive(Debug)]
pub enum UnitDBError {
    IOError(io::Error),
    ParseError(String),
}

impl From<io::Error> for UnitDBError {
    fn from(value: io::Error) -> Self {
        Self::IOError(value)
    }
}

impl From<String> for UnitDBError {
    fn from(value: String) -> Self {
        Self::ParseError(value)
    }
}

#[derive(Debug)]
pub struct UnitDB {
    /// map of nation id => nation name
    nations: HashMap<String, String>,
    /// map of vessel id => vessel
    vessels: HashMap<String, Vessel>,
    /// map of aircraft id => vessel
    aircraft: HashMap<String, Aircraft>,
}

impl UnitDB {
    pub fn new() -> Result<Self, UnitDBError> {
        let nations = load_nation_reference()?;
        let vessels = load_vessels()?;
        let aircraft = load_aircraft()?;
        Ok(Self { nations, vessels, aircraft })
    }

    pub fn nation_name(&self, id: &str) -> Option<&String> {
        self.nations.get(id)
    }

    pub fn all_vessels(&self) -> Vec<&Vessel> {
        self.vessels.values().collect()
    }

    pub fn vessel_by_id(&self, id: &str) -> Option<&Vessel> {
        self.vessels.get(id)
    }

    pub fn vessels_by_nation(&self, nation: &str) -> Vec<&Vessel> {
        self
            .all_vessels()
            .iter()
            .filter(|v| v.nation == nation)
            .map(|v| *v)
            .collect()
    }

    pub fn all_aircraft(&self) -> Vec<&Aircraft> {
        self.aircraft.values().collect()
    }

    pub fn aircraft_by_id(&self, id: &str) -> Option<&Aircraft> {
        self.aircraft.get(id)
    }
}
