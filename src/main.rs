mod dir;
mod gui;
mod mission;
mod rand_ext;
mod unit_db;

use configparser::ini::Ini;
use mission::Mission;
use std::error::Error;
use std::str;
use unit_db::UnitDb;

const MISSION_TEMPLATE: &str = include_str!("../resources/mission_template.ini");

fn load_template() -> Result<Ini, String> {
    let mut config = Ini::new_cs();
    config.read(MISSION_TEMPLATE.into())?;
    Ok(config)
}

fn main() -> Result<(), Box<dyn Error>> {
    // TODO: consider using channels to send missions instead
    gui::start(|options| {
        let unit_db = UnitDb::new().expect("failed to initialise UnitBD");
        let mission = Mission::new(&unit_db, options);
        eprintln!("{:#?}", mission);

        let mut config = load_template().expect("load template failed");
        mission.write_ini(&mut config);
        eprintln!("========== TEMPLATE ==========\n{}", config.writes());

        let mission_path = dir::mission_dir().join("Random Mission.ini");
        config.write(mission_path).expect("config write failed");
    });

    Ok(())
}
