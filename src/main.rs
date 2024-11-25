mod dir;
mod gen;
mod taskforce;
mod unit_db;

use configparser::ini::{Ini, WriteOptions};
use gen::GenOption;
use std::error::Error;
use std::path::Path;
use std::str;
use taskforce::{Taskforce, TaskforceOptions};
use unit_db::{UnitDB, Unit};

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
    n_neutral: GenOption,
    /// number of friendlies
    n_blue: GenOption,
    /// number of hostiles
    n_red: GenOption,
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
    println!("{:?}", unit_db);

    let mission_path = dir::mission_dir().join("Random Mission.ini");
    let mut config = load_template()?;

    let mission = Mission::new(MissionOptions {
        latlon: (54.0, -19.0),
        size: (100, 100),
        n_neutral: GenOption::MinMax(1, 5),
        n_blue: GenOption::Fixed(5),
        n_red: GenOption::Fixed(5),
    });
    println!("config: {:?}", mission);

    mission.write_environment(&mut config);

    let neutrals = gen::gen_neutrals(&mission.options.n_neutral, &unit_db);
    {
        let options = TaskforceOptions {
            weapon_state: taskforce::WeaponState::Hold,
            use_formation: false,
        };
        let taskforce = Taskforce::from_vec("Neutral", options, &neutrals);
        taskforce.write_config(&mut config, &mission);
        println!("{}", taskforce);
    }

    let blues = gen::gen_blues(&mission.options.n_blue, &unit_db);
    {
        let options = TaskforceOptions {
            weapon_state: taskforce::WeaponState::Tight,
            use_formation: true,
        };
        let taskforce = Taskforce::from_vec("Taskforce1", options, &blues);
        taskforce.write_config(&mut config, &mission);
        println!("{}", taskforce);
    }

    let reds = gen::gen_reds(&mission.options.n_red, &unit_db);
    {
        let options = TaskforceOptions {
            weapon_state: taskforce::WeaponState::Free,
            use_formation: true,
        };
        let taskforce = Taskforce::from_vec("Taskforce2", options, &reds);
        taskforce.write_config(&mut config, &mission);
        println!("{}", taskforce);
    }

    write_config(&mission_path, config)?;

    Ok(())
}
