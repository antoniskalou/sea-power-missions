mod dir;
mod gui;
mod mission;
mod rand_ext;
mod unit_db;

use configparser::ini::Ini;
use mission::Mission;
use std::error::Error;
use std::str;
use std::sync::Arc;
use unit_db::UnitDb;

const MISSION_TEMPLATE: &str = include_str!("../resources/mission_template.ini");

fn load_template() -> Result<Ini, String> {
    let mut config = Ini::new_cs();
    config.read(MISSION_TEMPLATE.into())?;
    Ok(config)
}

fn main() -> Result<(), Box<dyn Error>> {
    let unit_db = Arc::new(UnitDb::new().expect("failed to initialise UnitDB"));

    gui::start(&unit_db.all(), {
        let unit_db = unit_db.clone();
        move |options| {
            let mission = Mission::new(&unit_db.clone(), options);
            eprintln!("{:#?}", mission);

            // FIXME: give for useful errors to user
            let mut config = load_template().expect("load template failed");
            mission.write_ini(&mut config);

            let mission_path = dir::mission_dir().join("Random Mission.ini");
            config.write(mission_path).expect("config write failed");
        }
    });

    Ok(())
}
