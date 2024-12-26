use std::collections::HashMap;

use crate::dir;
use configparser::ini::Ini;
use std::{fs, io, path::Path};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum UnitType {
    Ship,
    Submarine,
    Helicopter,
    FixedWing,
    // TODO: figure out if this is necessary,
    // also consider using Option<VesselType>
    Unknown,
}

impl UnitType {
    pub fn capitalised_singular(&self) -> String {
        let str = match self {
            Self::Ship => "Vessel",
            Self::Submarine => "Submarine",
            Self::FixedWing => "Aircraft",
            Self::Helicopter => "Helicopter",
            Self::Unknown => panic!("unknown UnitType can not be coverted to string"),
        };
        str.to_owned()
    }

    pub fn calitalised_plural(&self) -> String {
        if let Self::FixedWing = self {
            "Aircraft".to_owned()
        } else {
            format!("{}s", self.capitalised_singular())
        }
    }
}

impl From<String> for UnitType {
    fn from(value: String) -> Self {
        match value.to_lowercase().as_str() {
            "vessel" => Self::Ship,
            "submarine" => Self::Submarine,
            "helicopter" => Self::Helicopter,
            "aircraft" => Self::FixedWing,
            _ => Self::Unknown,
        }
    }
}

impl std::fmt::Display for UnitType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.capitalised_singular())
    }
}

#[derive(Clone, Debug)]
pub struct Nation {
    pub id: String,
    pub name: String,
}

pub type UnitId = String;

#[derive(Clone, Debug)]
pub struct Unit {
    pub id: UnitId,
    pub name: String,
    pub nation: Nation,
    pub utype: UnitType,
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

fn load_nation_reference() -> Result<HashMap<String, Nation>, UnitDbError> {
    let config = load_ini(&dir::original_dir().join("nations_reference.ini"))?;

    let mut nations = HashMap::new();
    if let Some(map) = config.get_map() {
        for (_, nation) in map {
            let id = nation.get("nationprefix").and_then(|o| (*o).clone());
            let name = nation.get("nationname").and_then(|o| (*o).clone());
            if let Some((id, name)) = id.zip(name) {
                let nation = Nation {
                    id: id.clone(),
                    name,
                };
                nations.insert(id, nation);
            }
        }
    }
    Ok(nations)
}

fn split_name_parts(name: &str) -> Vec<&str> {
    name.split(",").collect()
}

fn load_vessel_names() -> Result<HashMap<String, String>, UnitDbError> {
    let config = load_ini(&dir::original_dir().join("language_en/vessel_names.ini"))?;
    let mut names = HashMap::new();
    if let Some(map) = config.get_map() {
        for (id, config) in map {
            let default = config.get("default").and_then(|o| (*o).clone());
            if let Some(name_parts) = default {
                let name = split_name_parts(&name_parts)[0];
                names.insert(id, name.to_string());
            }
        }
    }
    Ok(names)
}

fn load_vessels(nations: &HashMap<String, Nation>) -> Result<HashMap<String, Unit>, UnitDbError> {
    let names = load_vessel_names()?;
    let mut vessels = HashMap::new();

    for entry in fs::read_dir(dir::vessel_dir())? {
        let path = entry?.path();
        let id = match path_to_id(&path) {
            // skip storing variants for now, TODO
            Some(id) if !id.ends_with("_variants") => id.to_string(),
            _ => continue // skip invalid or variant ID's
        };

        let (nation_id, _) = match id.split_once("_") {
            Some(split) => split,
            None => continue,
        };

        // skip vessels without names, if they don't have one, they're
        // probably not important enough to include.
        let name = match names.get(&id) {
            Some(id) => id.to_string(),
            _ => continue
        };

        let nation = match nations.get(nation_id) {
            Some(nation) => nation.clone(),
            None => continue, // skip invalid nations
        };

        let config = load_ini(&path)?;
        let utype = config
            .get("General", "UnitType")
            .map(UnitType::from)
            .unwrap_or(UnitType::Unknown);

        vessels.insert(id.clone(), Unit {id, name, nation, utype, });
    }
    Ok(vessels)
}

fn load_aircraft() -> Result<HashMap<String, Unit>, UnitDbError> {
    Ok(HashMap::new())
}

#[derive(Debug)]
pub enum UnitDbError {
    IOError(io::Error),
    ParseError(String),
}

impl From<io::Error> for UnitDbError {
    fn from(value: io::Error) -> Self {
        Self::IOError(value)
    }
}

impl From<String> for UnitDbError {
    fn from(value: String) -> Self {
        Self::ParseError(value)
    }
}

#[derive(Debug)]
pub struct UnitDb {
    /// map of nation id => nation name
    nations: HashMap<String, Nation>,
    // map of unit id => unit
    units: HashMap<String, Unit>,
}

impl UnitDb {
    pub fn new() -> Result<Self, UnitDbError> {
        let nations = load_nation_reference()?;
        let mut units = HashMap::new();
        units.extend(load_vessels(&nations)?);
        units.extend(load_aircraft()?);
        Ok(Self { nations, units })
    }

    pub fn nations(&self) -> Vec<&Nation> {
        self.nations.values().collect()
    }

    pub fn all(&self) -> Vec<&Unit> {
        self.units.values().collect()
    }

    pub fn by_id(&self, id: &str) -> Option<&Unit> {
        self.units.get(id)
    }

    pub fn search(&self, nation: Option<&str>, utype: Option<UnitType>) -> Vec<&Unit> {
        self.all()
            .iter()
            .filter(|v| nation.map(|n| v.nation.id == n).unwrap_or(true))
            .filter(|v| utype.map(|s| v.utype == s).unwrap_or(true))
            .copied()
            .collect()
    }
}
