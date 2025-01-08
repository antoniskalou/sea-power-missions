mod config;
mod dir;
mod gui;
mod mission;
mod rand_ext;
mod unit_db;

use config::Config;
use configparser::ini::Ini;
use cursive::view::Nameable;
use cursive::views::{Dialog, EditView, LinearLayout, TextView};
use mission::Mission;
use std::error::Error;
use std::path::PathBuf;
use std::str;
use std::sync::{mpsc, Arc, Mutex};
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

fn ask_for_game_path() -> Config {
    let (tx, rx) = mpsc::sync_channel(1);

    let mut siv = cursive::default();
    siv.set_window_title("Sea Power Location Picker");
    siv.add_layer(
        Dialog::around(
            LinearLayout::vertical()
                .child(TextView::new(
                    "Failed to find your Sea Power install, please paste it below...",
                ))
                .child(EditView::new().with_name("path")),
        )
        .button("Ok", {
            let tx = tx.clone();
            move |s| {
                let content = s.call_on_name("path", |v: &mut EditView| v.get_content());
                tx.send(content).unwrap();
                s.quit();
            }
        })
        .title("Game location not found"),
    );
    siv.run();

    let root = rx.recv().unwrap();
    // FIXME: root doesn't actually check if path is correct
    Config::new(root.expect("game root not provided").as_ref())
}

fn main() -> Result<(), Box<dyn Error>> {
    // try and load the config...
    let (config, needs_write) = load_config()
        .map(|config| (config, false))
        // otherwise try and find the root directory
        .or_else(|| dir::find_root_dir().map(|path| (Config::new(path), true)))
        // can't find it, let's ask the user
        .unwrap_or_else(|| (ask_for_game_path(), true));
    eprintln!("{:#?}", config);
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

    // TODO: if not found, create a dialog with cursive and ask for input, then
    // update the game config file

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
