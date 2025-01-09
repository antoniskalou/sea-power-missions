mod config;
mod dir;
mod gui;
mod mission;
mod rand_ext;
mod unit_db;

use config::Config;
use configparser::ini::Ini;
use gui::AskForGamePathCommand;
use mission::Mission;
use std::error::Error;
use std::path::PathBuf;
use std::str;
use std::sync::Arc;
use unit_db::UnitDb;

const MISSION_TEMPLATE: &str = include_str!("../resources/mission_template.ini");

fn load_template() -> Result<Ini, String> {
    let mut config = Ini::new_cs();
    config.read(MISSION_TEMPLATE.into())?;
    Ok(config)
}

fn config_file() -> PathBuf {
    dir::config_dir().join("config.ini")
}

fn load_config() -> Option<Config> {
    let config_file = config_file();
    eprintln!("attempting to load config from {}", config_file.display());

    Config::load(&config_file)
        .inspect_err(|e| eprintln!("\tFAILED: {}", e))
        .ok()
        // ignore if path doesn't actually exist, we can then find and reset
        // the path later (from user input or known locations)
        .filter(|config| config.game_root.exists())
}

/// Repeatedly ask the user for the game path until the end of time.
fn ask_for_path_repeatedly() -> PathBuf {
    // if we've tried before and failed we should show a validation error
    let mut show_error = false;
    loop {
        match gui::ask_for_game_path(show_error) {
            AskForGamePathCommand::GiveUp => std::process::exit(0),
            AskForGamePathCommand::Save(path) if path.exists() => return path,
            _ => {
                show_error = true;
                continue;
            }
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // try and load the config...
    let (config, needs_write) = load_config()
        .map(|config| (config, false))
        // otherwise try and find the root directory
        // .or_else(|| dir::find_root_dir().map(|path| (Config::new(path), true)))
        // can't find it, let's ask the user
        .unwrap_or_else(|| (Config::new(ask_for_path_repeatedly()), true));
    eprintln!(
        "Detected game folder: {}",
        config
            .game_root
            .to_str()
            .expect("invalid root directory path")
    );

    if needs_write {
        eprintln!("updating config file...");
        config.save(config_file())?;
    }

    let unit_db = Arc::new(UnitDb::new(&config.game_root).expect("failed to initialise UnitDB"));
    gui::App::new(&unit_db).run({
        let unit_db = unit_db.clone();
        move |options| {
            let mission = Mission::new(&unit_db.clone(), options);

            // FIXME: give for useful errors to user
            let mut mission_config = load_template().expect("load template failed");
            mission.write_ini(&mut mission_config);

            let mission_path = dir::mission_dir(&config.game_root).join("Random Mission.ini");
            mission_config
                .write(mission_path)
                .expect("config write failed");
        }
    });

    Ok(())
}
