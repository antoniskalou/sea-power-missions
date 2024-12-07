use std::collections::HashMap;

use configparser::ini::Ini;

use crate::unit_db::{UnitId, UnitType};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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

#[derive(Clone, Debug)]
pub enum UnitOption {
    Unit(UnitId),
    Random {
        nation: Option<String>,
        subtype: Option<UnitType>,
    },
}

#[derive(Debug)]
pub struct Unit {
    id: String,
    heading: u16,
    position: (f32, f32),
}

impl Unit {
    pub fn write_ini(&self, config: &mut Ini, section: &str) {
        config.set(&section, "type", Some(self.id.clone()));
        // speed setting
        config.set(&section, "Telegraph", Some(2.to_string()));
        // defaults to "Green"
        config.set(&section, "CrewSkill", Some("Trained".to_owned()));
        // defaults to "Depleted"
        config.set(&section, "Stores", Some("Full".to_owned()));

        config.set(&section, "Heading", Some(self.heading.to_string()));

        {
            let (x, y) = self.position;
            let position_str = format!("{y},0,{x}");
            config.set(&section, "RelativePositionInNM", Some(position_str));
        }
    }
}

#[derive(Clone, Debug)]
pub struct FormationOption {
    units: Vec<UnitOption>,
}

#[derive(Clone, Debug)]
pub struct TaskforceOptions {
    weapon_state: WeaponState,
    units: Vec<UnitOption>,
    formations: Vec<FormationOption>,
}

type UnitReference = (UnitType, usize);

#[derive(Debug)]
pub struct Taskforce {
    options: TaskforceOptions,
    name: String,
    units: HashMap<UnitType, Vec<Unit>>,
    formations: Vec<Vec<UnitReference>>,
}

impl Taskforce {
    pub fn new(name: &str, options: TaskforceOptions) -> Self {
        Self {
            options,
            name: name.to_owned(),
            units: HashMap::new(),
            formations: vec![],
        }
    }

    pub fn write_ini(&self, config: &mut Ini) {
        for (utype, units) in &self.units {
            // setup NumberOf<TYPE><TASKFORCE>
            config.set(
                "Mission",
                &format!("NumberOf{}{}", self.name, utype.calitalised_plural()),
                Some(units.len().to_string()),
            );

            // create <TASKFORCE><TYPE><ID> where <ID> is reset for each <TYPE>
            for (idx, unit) in units.iter().enumerate() {
                let utype = utype.capitalised_singular();
                let section = format!("{}{utype}{}", self.name, idx + 1);
                unit.write_ini(config, &section);
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

fn formation_str(taskforce: &str, formation: &Vec<UnitReference>) -> String {
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

#[derive(Debug)]
struct MissionOptions {
    /// the map center latitude and logitude
    latlon: (f32, f32),
    /// the size of the box (w,h) that the mission will take place in.
    size: (u16, u16),
    neutral: TaskforceOptions,
    blue: TaskforceOptions,
    red: TaskforceOptions,
}

#[derive(Debug)]
struct Mission {
    options: MissionOptions,
    neutral: Taskforce,
    blue: Taskforce,
    red: Taskforce,
}

impl Mission {
    pub fn new(options: MissionOptions) -> Self {
        let neutral = Taskforce::new("Neutral", options.neutral.clone());
        let blue = Taskforce::new("Taskforce1", options.blue.clone());
        let red = Taskforce::new("Taskforce2", options.red.clone());
        Self { options, neutral, blue, red }
    }

    pub fn write_ini(&self, config: &mut Ini) {
        self.write_environment(config);
        self.neutral.write_ini(config);
        self.blue.write_ini(config);
        self.red.write_ini(config);
    }

    fn write_environment(&self, config: &mut Ini) {
        config.set(
            "Environment",
            "MapCenterLatitude",
            Some(self.options.latlon.0.to_string())
        );
        config.set(
            "Environment",
            "MapCenterLongitude",
            Some(self.options.latlon.1.to_string())
        );
    }
}

#[test]
fn test_mission_new() {
    let mission = Mission::new(MissionOptions {
        latlon: (12., 12.),
        size: (100, 100),
        neutral: TaskforceOptions {
            weapon_state: WeaponState::Hold,
            units: vec![
                UnitOption::Unit("civ_ms_kommunist".to_owned()),
            ],
            formations: vec![],
        },
        blue: TaskforceOptions {
            weapon_state: WeaponState::Tight,
            units: vec![
                UnitOption::Unit("wp_rkr_kirov".to_owned()),
            ],
            formations: vec![
                FormationOption {
                    units: vec![
                        UnitOption::Unit("wp_rkr_kirov".to_owned()),
                    ]
                }
            ],
        },
        red: TaskforceOptions {
            weapon_state: WeaponState::Free,
            units: vec![
                UnitOption::Random { nation: Some("usn".to_owned()), subtype: None }
            ],
            formations: vec![],
        }
    });
    println!("{:?}", mission);
}
