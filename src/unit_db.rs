use std::{collections::HashMap, path::PathBuf};

use crate::dir;
use configparser::ini::Ini;
use std::{fs, io, path::Path};
use thiserror::Error;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum UnitType {
    Ship,
    Submarine,
    Helicopter,
    FixedWing,
}

impl UnitType {
    pub fn all() -> [Self; 4] {
        use UnitType::*;
        [Ship, Submarine, Helicopter, FixedWing]
    }

    pub fn capitalised_singular(&self) -> String {
        let str = match self {
            Self::Ship => "Vessel",
            Self::Submarine => "Submarine",
            Self::FixedWing => "Aircraft",
            Self::Helicopter => "Helicopter",
        };
        str.to_owned()
    }

    pub fn calitalised_plural(&self) -> String {
        if *self == Self::FixedWing {
            "Aircraft".to_owned()
        } else {
            format!("{}s", self.capitalised_singular())
        }
    }
}

#[derive(Debug, Error)]
#[error("unknown unit type {0}")]
pub struct UnknownUnitTypeError(String);

impl TryFrom<String> for UnitType {
    type Error = UnknownUnitTypeError;

    fn try_from(utype: String) -> Result<Self, Self::Error> {
        match utype.to_lowercase().as_str() {
            "vessel" => Ok(Self::Ship),
            "submarine" => Ok(Self::Submarine),
            "helicopter" => Ok(Self::Helicopter),
            "aircraft" => Ok(Self::FixedWing),
            _ => Err(UnknownUnitTypeError(utype.to_owned())),
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

impl std::fmt::Display for Nation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // FIXME: it would be nice to provide a {name} ({id}), but I can't since it breaks
        // filter. Code from filter should not depend on the string intepretation.
        write!(f, "{}", self.name)
    }
}

// normally this wouldn't be required, since std::fmt::Display covers that use-case,
// but cursive_tree_view insists of needing this
impl From<&Nation> for String {
    fn from(value: &Nation) -> Self {
        value.to_string()
    }
}

pub type UnitId = String;

#[derive(Clone, Debug)]
pub struct Unit {
    pub id: UnitId,
    pub name: String,
    pub nation: Nation,
    pub utype: UnitType,
}

#[derive(Error, Debug)]
pub enum UnitDbError {
    #[error("failed to parse ini file {file}: {reason}")]
    ParseError { file: PathBuf, reason: String },
    #[error(transparent)]
    IOError(#[from] io::Error),
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
        let mut nations: Vec<&Nation> = self.nations.values().collect();
        nations.sort_by(|a, b| a.id.cmp(&b.id));
        nations
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

/// Sea Power encodes unit information in the filename, usually structured
/// like <nation>_<vessel_name>
fn path_to_id(path: &Path) -> Option<&str> {
    path.file_stem().and_then(|p| p.to_str())
}

/// load a unit ini file
fn load_ini(path: &Path) -> Result<Ini, UnitDbError> {
    let mut config = Ini::new();

    match config.load(path) {
        Ok(_) => Ok(config),
        Err(reason) => Err(UnitDbError::ParseError {
            file: path.to_owned(),
            reason,
        }),
    }
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
            _ => continue, // skip invalid or variant ID's
        };

        let (nation_id, _) = match id.split_once("_") {
            Some(split) => split,
            None => continue,
        };

        // skip vessels without names, if they don't have one, they're
        // probably not important enough to include.
        let name = match names.get(&id) {
            Some(id) => id.to_string(),
            _ => continue,
        };

        let nation = match nations.get(nation_id) {
            Some(nation) => nation.clone(),
            None => continue, // skip invalid nations
        };

        let config = load_ini(&path)?;

        let utype = match config
            .get("General", "UnitType")
            .and_then(|utype| UnitType::try_from(utype).ok())
        {
            Some(utype) => utype,
            None => continue, // skip invalid types
        };

        vessels.insert(
            id.clone(),
            Unit {
                id,
                name,
                nation,
                utype,
            },
        );
    }
    Ok(vessels)
}

fn load_aircraft() -> Result<HashMap<String, Unit>, UnitDbError> {
    Ok(HashMap::new())
}
