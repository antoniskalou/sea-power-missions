use std::collections::HashMap;

use configparser::ini::Ini;
use crate::{unit_db::Unit, Mission};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum WeaponState {
    Free,
    Tight,
    Hold,
}

impl ToString for WeaponState {
    fn to_string(&self) -> String {
        use WeaponState::*;
        let str = match self {
            Free => "Free",
            Tight => "Tight",
            Hold => "Hold",
        };
        str.to_owned()
    }
}

#[derive(Debug)]
pub struct TaskforceOptions {
    pub weapon_state: WeaponState,
}

type Formation = Vec<u32>;

/// A taskforce represents a "side" of the engagement. They can be "Neutral"
/// or "Taskforce1" and "Taskforce2".
///
/// It can contain formations and individual units.
#[derive(Debug)]
pub struct Taskforce {
    name: String,
    options: TaskforceOptions,
    last_unit_id: u32,
    units: HashMap<u32, Unit>,
    formations: Vec<Formation>,
}

impl Taskforce {
    pub fn new(name: &str, options: TaskforceOptions) -> Self {
        Self {
            name: name.to_owned(),
            options,
            // IDs start from 1 in the ini files
            last_unit_id: 1,
            formations: vec![],
            units: HashMap::new(),
        }
    }

    pub fn add(&mut self, unit: &Unit) {
        self.add_with_id(unit);
    }

    pub fn add_formation(&mut self, units: &Vec<Unit>) {
        let ids = units.iter()
            .map(|u| self.add_with_id(u))
            .collect();
        self.formations.push(ids);
    }

    fn unit_id(&mut self) -> u32 {
        let id = self.last_unit_id;
        self.last_unit_id += 1;
        id
    }

    fn add_with_id(&mut self, unit: &Unit) -> u32 {
        let id = self.unit_id();
        self.units.insert(id, unit.clone());
        id
    }

    pub fn write_config(&self, config: &mut Ini, mission: &Mission) {
        config.set(
            "Mission",
            // FIXME: don't use Vessel, instead determine type
            &format!("NumberOf{}Vessels", self.name),
            Some(self.units.len().to_string()),
        );

        // TODO: for now units are all either in a formation or they are not
        // there is no way to specify how many formations should exist and
        // how large they should be (yet).
        config.set(
            "Mission",
            &format!("{}_NumberOfFormations", self.name),
            // TODO: figure out if a len of 0 will present a problem
            Some(self.formations.len().to_string()),
        );
        for (i, formation) in self.formations.iter().enumerate() {
            config.set(
                "Mission",
                &format!("{}_Formation{}", self.name, i + 1),
                Some(formation_str(&self.name, formation))
            );
        }

        for (id, vessel) in &self.units {
            let section = format!("{}Vessel{}", self.name, id);
            mission.write_unit(config, &section, &vessel);
            config.set(
                &section,
                "WeaponStatus",
                Some(self.options.weapon_state.to_string()),
            );
        }
    }
}

impl std::fmt::Display for Taskforce {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]\n", self.name)?;

        let mut units = self.units.clone();
        for (i, formation) in self.formations.iter().enumerate() {
            write!(f, "Formation {}\n", i + 1)?;
            for id in formation {
                let unit = units.remove(id).unwrap();
                write!(f, "\t{} ==> {}\n", id, unit.id)?;
            }
        }

        let mut sorted_keys = units.keys().collect::<Vec<_>>();
        sorted_keys.sort();
        // whatever is left is a singular unit
        for id in sorted_keys {
            // unwrap never fails
            let unit = units.get(id).unwrap();
            write!(f, "{} ==> {}\n", id, unit.id)?;
        }

        Ok(())
    }
}

fn formation_str(taskforce: &str, formation: &Formation) -> String {
    let sections = formation.iter()
        .map(|id| format!("{}Vessel{}", taskforce, id))
        .collect::<Vec<_>>();
    let formation = sections.join(",");
    // OverrideSpawnPositions allows us to place our units anywhere and the formation
    // will be adjusted by the game on mission start (which is exactly what we want)
    format!("{formation}|Unnamed Group|Circle|1.5|OverrideSpawnPositions")
}
