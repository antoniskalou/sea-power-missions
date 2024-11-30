use std::collections::HashMap;
use configparser::ini::Ini;
use crate::{gen::UnitOrFormation, unit_db::{Unit, UnitType}, Mission};

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

type Formation = Vec<(UnitType, usize)>;

/// A taskforce represents a "side" of the engagement. They can be "Neutral"
/// or "Taskforce1" and "Taskforce2".
///
/// It can contain formations and individual units.
#[derive(Debug)]
pub struct Taskforce {
    name: String,
    options: TaskforceOptions,
    units: HashMap<UnitType, Vec<Unit>>,
    formations: Vec<Formation>,
}

impl Taskforce {
    pub fn new(
        name: &str,
        units_or_formations: &Vec<UnitOrFormation>,
        options: TaskforceOptions
    ) -> Self {
        let mut units = HashMap::<UnitType, Vec<Unit>>::new();
        let mut formations = vec![];
        for unit in units_or_formations {
            match unit {
                UnitOrFormation::Unit(unit) => {
                    units.entry(unit.utype)
                        .and_modify(|units| units.push(unit.clone()))
                        .or_insert_with(|| vec![unit.clone()]);
                }
                UnitOrFormation::Formation(formation) => {
                    let ids = formation
                        .iter()
                        .map(|unit| {
                            let idx = units.get(&unit.utype)
                                .map(|v| v.len())
                                .unwrap_or(0);
                            units.entry(unit.utype)
                                .and_modify(|units| units.push(unit.clone()))
                                .or_insert_with(|| vec![unit.clone()]);
                            (unit.utype, idx)
                        })
                        .collect();
                    formations.push(ids);
                }
            }
        }

        Self {
            name: name.to_owned(),
            formations,
            options,
            units,
        }
    }

    pub fn write_config(&self, config: &mut Ini, mission: &Mission) {
        for (utype, units) in &self.units {
            // setup NumberOf<TYPE><TASKFORCE>
            config.set(
                "Mission",
                &format!("NumberOf{}{}", self.name, utype.calitalised_plural()),
                Some(units.len().to_string()),
            );

            // create <TASKFORCE><TYPE><ID> where <ID> is reset for each <TYPE>
            for (idx, unit) in units.iter().enumerate() {
                let utype = unit.utype.capitalised_singular();
                let section = format!("{}{utype}{}", self.name, idx + 1);
                mission.write_unit(config, &section, &unit);
                config.set(
                    &section,
                    "WeaponStatus",
                    Some(self.options.weapon_state.to_string()),
                );
            }
        }

        // create formations (not type specific)
        config.set(
            "Mission",
            &format!("{}_NumberOfFormations", self.name),
            Some(self.formations.len().to_string()),
        );
        for (f_idx, formation) in self.formations.iter().enumerate() {
            // formation can use any type of unit
            config.set(
                "Mission",
                &format!("{}_Formation{}", self.name, f_idx + 1),
                Some(formation_str(&self.name, formation))
            );
        }
    }
}

// impl std::fmt::Display for Taskforce {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "[{}]\n", self.name)?;

//         let mut units = self.units.clone();
//         for (utype, units) in self.formations.iter() {
//             write!(f, "Formation {}\n", i + 1)?;
//             for idx in units {
//                 let unit = units.remove(id).unwrap();
//                 write!(f, "\t{} ==> {}\n", id, unit.id)?;
//             }
//         }

//         let mut sorted_keys = units.keys().collect::<Vec<_>>();
//         sorted_keys.sort();
//         // whatever is left is a singular unit
//         for id in sorted_keys {
//             // unwrap never fails
//             let unit = units.get(id).unwrap();
//             write!(f, "{} ==> {}\n", id, unit.id)?;
//         }

//         Ok(())
//     }
// }

fn formation_str(taskforce: &str, formation: &Formation) -> String {
    let sections = formation.iter()
        .map(|(utype, idx)| {
            let utype = utype.capitalised_singular();
            format!("{taskforce}{utype}{}", idx + 1)
        })
        .collect::<Vec<_>>();
    let formation = sections.join(",");
    // OverrideSpawnPositions allows us to place our units anywhere and the formation
    // will be adjusted by the game on mission start (which is exactly what we want)
    format!("{formation}|Unnamed Group|Circle|1.5|OverrideSpawnPositions")
}
