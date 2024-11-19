mod dir;

use std::path::Path;
use configparser::ini::{Ini, IniDefault, WriteOptions};

const MISSION_TEMPLATE: &'static str = include_str!("../resources/mission_template.ini");

fn load_template() -> Result<Ini, String> {
    let mut default = IniDefault::default();
    default.case_sensitive = true;
    default.boolean_values = [
        (
            true,
            ["True", "true"]
                .iter()
                .map(|&s| s.to_owned())
                .collect(),
        ),
        (
            false,
            ["False", "false"]
                .iter()
                .map(|&s| s.to_owned())
                .collect(),
        ),
    ].iter().cloned().collect();
    let mut config = Ini::new_from_defaults(default);
    config.read(MISSION_TEMPLATE.to_owned())?;
    Ok(config)
}

fn write_template(path: &Path, config: Ini) -> std::io::Result<()> {
    println!("Writing to {:?}", path);
    config.pretty_write(path, &WriteOptions::default())
}

fn main() {
    println!("Does {:?} exist? {}", dir::mission_dir(), dir::mission_dir().exists());
    let mission_path = dir::mission_dir().join("Random Mission.ini");
    let config = load_template()
        .expect("failed to read mission_template.ini");
    println!("Loaded template: {:?}", config.get_map().unwrap());
    write_template(&mission_path, config)
        .expect("failed to write mission file");
}
