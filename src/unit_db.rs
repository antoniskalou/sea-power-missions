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
    pub fn all() -> [UnitType; 4] {
        use UnitType::*;
        [Ship, Submarine, Helicopter, FixedWing]
    }

    pub fn capitalised_singular(&self) -> String {
        let str = match self {
            Self::Ship => "Vessel",
            Self::Submarine => "Submarine",
            Self::FixedWing => "Aircraft",
            Self::Helicopter => "Helicopter",
            Self::Unknown =>
                panic!("unknown UnitType can not be coverted to string"),
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

pub type UnitId = String;

#[derive(Clone, Debug)]
pub struct Unit {
    pub id: UnitId,
    pub nation: String,
    pub subtype: UnitType,
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

// TODO: can load unit names from language_en/vessel_names.ini
fn load_nation_reference() -> Result<HashMap<String, String>, UnitDBError> {
    let config = load_ini(&dir::original_dir().join("language_en/nations.ini"))?;

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

fn load_vessels() -> Result<HashMap<String, Unit>, UnitDBError> {
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
                .map(|t| UnitType::from(t))
                .unwrap_or(UnitType::Unknown);
            vessels.insert(
                id.clone(),
                Unit { id, nation, subtype, },
            );
        }
    }
    Ok(vessels)
}

fn load_aircraft() -> Result<HashMap<String, Unit>, UnitDBError> {
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
    // map of unit id => unit
    units: HashMap<String, Unit>,
}

impl UnitDB {
    pub fn new() -> Result<Self, UnitDBError> {
        let nations = load_nation_reference()?;
        let mut units = HashMap::new();
        units.extend(load_vessels()?);
        units.extend(load_aircraft()?);
        Ok(Self { nations, units, })
    }

    pub fn nation_name(&self, id: &str) -> Option<&String> {
        self.nations.get(id)
    }

    pub fn all(&self) -> Vec<&Unit> {
        self.units.values().collect()
    }

    pub fn by_id(&self, id: &str) -> Option<&Unit> {
        self.units.get(id)
    }

    pub fn by_nation(&self, nation: &str) -> Vec<&Unit> {
        self.units
            .values()
            .filter(|v| v.nation == nation)
            .collect()
    }

    pub fn by_subtype(&self, subtype: UnitType) -> Vec<&Unit> {
        self.units
            .values()
            .filter(|v| v.subtype == subtype)
            .collect()
    }

    pub fn search(
        &self,
        nation: Option<&str>,
        subtype: Option<UnitType>
    ) -> Vec<&Unit> {
        self.all()
            .iter()
            .filter(|v| nation.map(|n| v.nation == n).unwrap_or(true))
            .filter(|v| subtype.map(|s| v.subtype == s).unwrap_or(true))
            .map(|v| *v)
            .collect()
    }
}
