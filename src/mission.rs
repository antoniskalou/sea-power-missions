use std::collections::HashMap;

use configparser::ini::Ini;
use rand::{seq::SliceRandom, thread_rng, Rng};

use crate::unit_db::{self, UnitDb, UnitId, UnitType};

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
        utype: Option<UnitType>,
    },
}

#[derive(Clone, Debug)]
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
            let position_str = format!("{x},0,{y}");
            config.set(&section, "RelativePositionInNM", Some(position_str));
        }
    }
}

#[derive(Clone, Debug)]
pub struct FormationOption {
    pub units: Vec<UnitOption>,
}

#[derive(Clone, Debug)]
pub struct TaskforceOptions {
    pub weapon_state: WeaponState,
    pub units: Vec<UnitOption>,
    pub formations: Vec<FormationOption>,
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
    pub fn new(unit_db: &UnitDb, name: &str, options: TaskforceOptions) -> Self {
        let mut units = HashMap::new();
        // insert lone units (outside of formation)
        insert_units(unit_db, &mut units, &options.units);

        let mut formations = Vec::new();
        for formation_opt in &options.formations {
            formations.push(insert_units(unit_db, &mut units, &formation_opt.units));
        }

        Self {
            options,
            name: name.to_owned(),
            units,
            formations,
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
                Some(formation_str(&self.name, formation)),
            );
        }
    }
}

fn insert_units(
    unit_db: &UnitDb,
    units: &mut HashMap<UnitType, Vec<Unit>>,
    unit_opts: &Vec<UnitOption>,
) -> Vec<UnitReference> {
    unit_opts
        .iter()
        .filter_map(|unit_opt| match unit_opt {
            UnitOption::Unit(id) => unit_db.by_id(&id).map(|unit| insert_unit(units, unit)),
            UnitOption::Random { nation, utype } => {
                let matches = unit_db.search(nation.as_deref(), *utype);
                matches
                    .choose(&mut thread_rng())
                    .map(|unit| insert_unit(units, unit))
            }
        })
        .collect()
}

fn insert_unit(units: &mut HashMap<UnitType, Vec<Unit>>, db_unit: &unit_db::Unit) -> UnitReference {
    let unit_list = units.entry(db_unit.utype).or_insert_with(Vec::new);
    let index = unit_list.len();
    unit_list.push(Unit {
        id: db_unit.id.clone(),
        heading: 0,
        position: (0., 0.),
    });
    (db_unit.utype, index)
}

fn formation_str(taskforce: &str, formation: &Vec<UnitReference>) -> String {
    let sections = formation
        .iter()
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

#[derive(Clone, Debug)]
pub struct MissionOptions {
    /// the map center latitude and logitude
    pub latlon: (f32, f32),
    /// the size of the box (w,h) that the mission will take place in.
    pub size: (u16, u16),
    pub neutral: TaskforceOptions,
    pub blue: TaskforceOptions,
    pub red: TaskforceOptions,
    // TODO: add environment config
}

#[derive(Debug)]
pub struct Mission {
    options: MissionOptions,
    neutral: Taskforce,
    blue: Taskforce,
    red: Taskforce,
}

impl Mission {
    pub fn new(unit_db: &UnitDb, options: MissionOptions) -> Self {
        let neutral = Taskforce::new(unit_db, "Neutral", options.neutral.clone());
        let blue = Taskforce::new(unit_db, "Taskforce1", options.blue.clone());
        let red = Taskforce::new(unit_db, "Taskforce2", options.red.clone());
        Self {
            options,
            neutral,
            blue,
            red,
        }
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
            Some(self.options.latlon.0.to_string()),
        );
        config.set(
            "Environment",
            "MapCenterLongitude",
            Some(self.options.latlon.1.to_string()),
        );
    }
}
