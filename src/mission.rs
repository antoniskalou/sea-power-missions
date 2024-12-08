use std::collections::HashMap;

use crate::rand_ext;
use configparser::ini::Ini;
use rand::{seq::SliceRandom, thread_rng};

use crate::unit_db::{self, UnitDb, UnitId, UnitType};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum WeaponState {
    Free,
    Tight,
    Hold,
}

impl std::fmt::Display for WeaponState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use WeaponState::*;
        let str = match self {
            Free => "Free",
            Tight => "Tight",
            Hold => "Hold",
        };
        write!(f, "{str}")
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
    pub fn new(general: &GeneralOptions, id: &str) -> Self {
        Self {
            id: id.to_owned(),
            heading: rand_ext::heading(),
            position: rand_ext::position(&general.size),
        }
    }

    pub fn write_ini(&self, config: &mut Ini, section: &str) {
        config.set(&section, "Type", Some(self.id.clone()));
        // speed setting
        config.set(&section, "Telegraph", Some(3.to_string()));
        // defaults to "Green"
        config.set(&section, "CrewSkill", Some("Trained".into()));
        // defaults to "Depleted"
        config.set(&section, "Stores", Some("Full".into()));

        // generate our own positions and headings, don't use RandomSpawn*
        // because by having static positions the player can replay the mission
        // or share it if they like it without changing the experience each
        // time the generator is run.
        //
        // It also means that the generated config can be used by the mission editor
        // without overwriting the Random* options (which currently happens in v0.1.0.6).
        config.set(&section, "Heading", Some(self.heading.to_string()));
        {
            let (x, y) = self.position;
            config.set(&section, "RelativePositionInNM", Some(format!("{x},0,{y}")));
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
    pub fn new(
        unit_db: &UnitDb,
        general: &GeneralOptions,
        name: &str,
        options: TaskforceOptions,
    ) -> Self {
        let mut units = HashMap::new();
        // insert lone units (outside of formation)
        insert_units(unit_db, &general, &mut units, &options.units);

        let mut formations = Vec::new();
        for formation_opt in &options.formations {
            formations.push(insert_units(
                unit_db,
                &general,
                &mut units,
                &formation_opt.units,
            ));
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
    general: &GeneralOptions,
    units: &mut HashMap<UnitType, Vec<Unit>>,
    unit_opts: &[UnitOption],
) -> Vec<UnitReference> {
    unit_opts
        .iter()
        .filter_map(|unit_opt| match unit_opt {
            UnitOption::Unit(id) => unit_db
                .by_id(&id)
                // TODO: fail if not found
                .map(|unit| insert_unit(general, units, unit)),
            UnitOption::Random { nation, utype } => {
                let matches = unit_db.search(nation.as_deref(), *utype);
                matches
                    .choose(&mut thread_rng())
                    .map(|unit| insert_unit(general, units, unit))
            }
        })
        .collect()
}

fn insert_unit(
    general: &GeneralOptions,
    units: &mut HashMap<UnitType, Vec<Unit>>,
    db_unit: &unit_db::Unit,
) -> UnitReference {
    let unit_list = units.entry(db_unit.utype).or_insert_with(Vec::new);
    let index = unit_list.len();
    unit_list.push(Unit::new(general, &db_unit.id));
    (db_unit.utype, index)
}

fn formation_str(taskforce: &str, formation: &[UnitReference]) -> String {
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
    //
    // use 7.5nm spacing for now, this may not be optimal in tight spaces, but this mod is currently
    // unable to handle those kind of scenarios anyway.
    format!("{formation}|Unnamed Group|Circle|7.5|OverrideSpawnPositions")
}

/// Mission wide options
#[derive(Clone, Debug)]
pub struct GeneralOptions {
    /// the map center latitude and logitude
    pub latlon: (f32, f32),
    /// the size of the box (w,h) that the mission will take place in.
    pub size: (u16, u16),
}

#[derive(Clone, Debug)]
pub struct MissionOptions {
    pub general: GeneralOptions,
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
        let neutral = Taskforce::new(
            unit_db,
            &options.general,
            "Neutral",
            options.neutral.clone(),
        );
        let blue = Taskforce::new(
            unit_db,
            &options.general,
            "Taskforce1",
            options.blue.clone(),
        );
        let red = Taskforce::new(unit_db, &options.general, "Taskforce2", options.red.clone());
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
        let (lat, lon) = self.options.general.latlon;
        config.set("Environment", "MapCenterLatitude", Some(lat.to_string()));
        config.set("Environment", "MapCenterLongitude", Some(lon.to_string()));
    }
}
