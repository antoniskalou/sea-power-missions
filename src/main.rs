mod dir;
mod gen;
mod taskforce;
mod unit_db;

use configparser::ini::{Ini, WriteOptions};
use std::error::Error;
use std::path::Path;
use std::str;
use taskforce::{Taskforce, TaskforceOptions};
use unit_db::{UnitDB, Unit, UnitId, UnitType};
use gen::UnitOption;

const MISSION_TEMPLATE: &'static str = include_str!("../resources/mission_template.ini");

fn load_template() -> Result<Ini, String> {
    let mut config = Ini::new_cs();
    config.read(MISSION_TEMPLATE.to_owned())?;
    Ok(config)
}

fn write_config(path: &Path, config: Ini) -> std::io::Result<()> {
    let mut options = WriteOptions::default();
    options.blank_lines_between_sections = 1;
    config.pretty_write(path, &options)
}

#[derive(Debug)]
struct MissionOptions {
    /// the map center latitude and logitude
    latlon: (f32, f32),
    /// the size of the box (w,h) that the mission will take place in.
    size: (u16, u16),
    /// the maximum number of neutrals to generate
    neutral: Vec<UnitOption>,
    /// number of friendlies
    blue: Vec<UnitOption>,
    /// number of hostiles
    red: Vec<UnitOption>,
}

#[derive(Debug)]
struct Mission {
    options: MissionOptions,
}

impl Mission {
    fn new(options: MissionOptions) -> Self {
        Self { options }
    }

    // TODO: move me
    fn write_unit(&self, config: &mut Ini, section: &str, unit: &Unit) {
        config.set(&section, "type", Some(unit.id.clone()));
        // speed setting
        config.set(&section, "Telegraph", Some(2.to_string()));
        // defaults to "Green"
        config.set(&section, "CrewSkill", Some("Trained".to_owned()));
        // defaults to "Depleted"
        config.set(&section, "Stores", Some("Full".to_owned()));

        let position = gen::gen_position(&self.options.size);
        config.set(&section, "RelativePositionInNM", Some(position.to_string()));

        let heading = gen::gen_heading();
        config.set(&section, "Heading", Some(heading.to_string()));
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

fn main() -> Result<(), Box<dyn Error>> {
    let unit_db = UnitDB::new().expect("failed to initialise UnitBD");
    // println!("{:?}", unit_db);

    let mission_path = dir::mission_dir().join("Random Mission.ini");
    let mut config = load_template()?;

    let mission = Mission::new(MissionOptions {
        latlon: (54.0, -19.0),
        size: (100, 100),
        neutral: vec![
            UnitOption::Random {
                nation: Some("civ".to_owned()),
                subtype: None,
            },
            UnitOption::Random {
                nation: Some("civ".to_owned()),
                subtype: None
            },
            UnitOption::Random {
                nation: Some("civ".to_owned()),
                subtype: None
            },
            UnitOption::Random {
                nation: Some("civ".to_owned()),
                subtype: None
            },
        ],
        blue: vec![
            UnitOption::Formation(vec![
                UnitOption::Random {
                    nation: Some("wp".to_owned()),
                    subtype: Some(UnitType::Ship),
                },
                UnitOption::Unit(UnitId::from("wp_bpk_udaloy")),
                UnitOption::Unit(UnitId::from("wp_rkr_kirov")),
            ]),
            UnitOption::Formation(vec![
                UnitOption::Random {
                    nation: Some("wp".to_owned()),
                    subtype: Some(UnitType::Ship),
                },
                UnitOption::Random {
                    nation: Some("wp".to_owned()),
                    subtype: Some(UnitType::Ship),
                },
            ]),
            UnitOption::Random {
                nation: Some("wp".to_owned()),
                subtype: Some(UnitType::Submarine),
            },
            UnitOption::Random {
                nation: Some("wp".to_owned()),
                subtype: Some(UnitType::Submarine),
            }
        ],
        red: vec![
            UnitOption::Formation(vec![
                UnitOption::Random {
                    nation: Some("usn".to_owned()),
                    subtype: None,
                },
                UnitOption::Unit(UnitId::from("usn_cg_belknap")),
                UnitOption::Unit(UnitId::from("usn_cv_kitty_hawk")),
            ]),
            UnitOption::Random {
                nation: Some("usn".to_owned()),
                subtype: Some(UnitType::Submarine),
            }
        ],
    });

    mission.write_environment(&mut config);

    {
        let options = TaskforceOptions {
            weapon_state: taskforce::WeaponState::Hold,
        };
        let mut taskforce = Taskforce::new("Neutral", options);
        gen::gen_taskforce(&mut taskforce, &unit_db, &mission.options.neutral);
        taskforce.write_config(&mut config, &mission);
        println!("{:?}", taskforce);
    }

    {
        let options = TaskforceOptions {
            weapon_state: taskforce::WeaponState::Tight,
        };
        let mut taskforce = Taskforce::new("Taskforce1", options);
        gen::gen_taskforce(&mut taskforce, &unit_db, &mission.options.blue);
        taskforce.write_config(&mut config, &mission);
        println!("{:?}", taskforce);
    }

    {
        let options = TaskforceOptions {
            weapon_state: taskforce::WeaponState::Free,
        };
        let mut taskforce = Taskforce::new("Taskforce2", options);
        gen::gen_taskforce(&mut taskforce, &unit_db, &mission.options.red);
        taskforce.write_config(&mut config, &mission);
        println!("{:?}", taskforce);
    }

    write_config(&mission_path, config)?;

    Ok(())
}
