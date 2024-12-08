mod dir;
mod rand_ext;
mod mission;
mod unit_db;

use configparser::ini::{Ini, WriteOptions};
use std::error::Error;
use std::path::Path;
use std::str;
use mission::{
    FormationOption, GeneralOptions, Mission, MissionOptions, TaskforceOptions, UnitOption,
    WeaponState,
};
use unit_db::UnitDb;

const MISSION_TEMPLATE: &'static str = include_str!("../resources/mission_template.ini");

fn load_template() -> Result<Ini, String> {
    let mut config = Ini::new_cs();
    config.read(MISSION_TEMPLATE.into())?;
    Ok(config)
}

fn save_config(path: &Path, config: Ini) -> std::io::Result<()> {
    let mut options = WriteOptions::default();
    options.blank_lines_between_sections = 1;
    config.pretty_write(path, &options)
}

fn main() -> Result<(), Box<dyn Error>> {
    let unit_db = UnitDb::new().expect("failed to initialise UnitBD");
    // println!("{:?}", unit_db);

    let mission = Mission::new(
        &unit_db,
        MissionOptions {
            general: GeneralOptions {
                latlon: (34.31, 29.62),
                size: (100, 100),
            },
            neutral: TaskforceOptions {
                weapon_state: WeaponState::Hold,
                units: vec![UnitOption::Unit("civ_ms_kommunist".into())],
                formations: vec![],
            },
            blue: TaskforceOptions {
                weapon_state: WeaponState::Tight,
                units: vec![UnitOption::Unit("wp_rkr_kirov".into())],
                formations: vec![FormationOption {
                    units: vec![UnitOption::Unit("wp_rkr_kirov".into())],
                }],
            },
            red: TaskforceOptions {
                weapon_state: WeaponState::Free,
                units: vec![UnitOption::Random {
                    nation: Some("usn".into()),
                    utype: None,
                }],
                formations: vec![],
            },
        },
    );
    println!("{:#?}", mission);

    let mut config = load_template()?;
    mission.write_ini(&mut config);
    println!("========== TEMPLATE ==========\n{}", config.writes());

    let mission_path = dir::mission_dir().join("Random Mission.ini");
    // let mission_path = Path::new("./mission.ini");
    save_config(&mission_path, config)?;

    Ok(())
}
