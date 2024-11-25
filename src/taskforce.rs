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
    pub use_formation: bool,
}

/// A taskforce represents a "side" of the engagement. They can be "Neutral"
/// or "Taskforce1" and "Taskforce2".
///
/// It can contain formations and individual units.
#[derive(Debug)]
pub struct Taskforce {
    name: String,
    options: TaskforceOptions,
    units: Vec<Unit>,
}

impl Taskforce {
    pub fn new(name: &str, options: TaskforceOptions) -> Self {
        Self::from_vec(name, options, &vec![])
    }

    pub fn from_vec(name: &str, options: TaskforceOptions, units: &Vec<&Unit>) -> Self {
        Self {
            name: name.to_owned(),
            // TODO: allow providing options
            options,
            units: units.iter().map(|v| (*v).clone()).collect::<Vec<_>>(),
        }
    }

    pub fn add(&mut self, vessel: &Unit) {
        self.units.push(vessel.clone());
    }

    pub fn write_config(&self, config: &mut Ini, mission: &Mission) {
        let n = self.units.len();
        config.set(
            "Mission",
            // FIXME: don't use Vessel, instead determine type
            &format!("NumberOf{}Vessels", self.name),
            Some(n.to_string()),
        );

        // TODO: for now units are all either in a formation or they are not
        // there is no way to specify how many formations should exist and
        // how large they should be (yet).
        if self.options.use_formation {
            config.set(
                "Mission",
                &format!("{}_NumberOfFormations", self.name),
                Some(1.to_string()),
            );
            config.set(
                "Mission",
                &format!("{}_Formation1", self.name),
                Some(formation_str(&self)),
            );
        }

        for (i, vessel) in self.units.iter().enumerate() {
            let section = format!("{}Vessel{}", self.name, i + 1);
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
        let vessels = self.units
            .iter()
            .map(|v| format!("==> {}", v.id.clone()))
            .collect::<Vec<_>>()
            .join("\n");
        write!(f, "{}\n{}", self.name, vessels)
    }
}

fn formation_str(taskforce: &Taskforce) -> String {
    let n = taskforce.units.len();
    let sections = (0..n)
        .map(|i| format!("{}Vessel{}", taskforce.name, i + 1))
        .collect::<Vec<_>>();
    let formation = sections.join(",");
    // OverrideSpawnPositions allows us to place our units anywhere and the formation
    // will be adjusted by the game on mission start (which is exactly what we want)
    format!("{formation}|Group Name 1|Circle|1.5|OverrideSpawnPositions")
}
