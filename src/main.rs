mod dir;

use rand::{thread_rng, Rng};
use std::path::Path;
use std::str;
use configparser::ini::{Ini, WriteOptions};

const MISSION_TEMPLATE: &'static str = include_str!("../resources/mission_template.ini");

fn load_template() -> Result<Ini, String> {
    let mut config = Ini::new_cs();
    config.read(MISSION_TEMPLATE.to_owned())?;
    Ok(config)
}

fn write_template(path: &Path, config: Ini) -> std::io::Result<()> {
    let mut options = WriteOptions::default();
    options.blank_lines_between_sections = 1;
    config.pretty_write(path, &options)
}

#[derive(Clone, Debug)]
struct Vessel {
    id: String,
    nation: String,
}

fn path_to_id(path: &Path) -> Option<&str> {
    path.file_stem().and_then(|p| p.to_str())
}

fn load_vessels() -> std::io::Result<Vec<Vessel>> {
    let entries = std::fs::read_dir(dir::vessel_dir())?
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_file());

    let vessels = entries
        // ignore _variants file
        .filter(|path| {
            !path_to_id(path)
                // safe to unwrap since we know that the path is definitely a file
                .unwrap()
                .ends_with("_variants")
        })
        .filter_map(|path| {
            let id = path_to_id(&path).unwrap();
            id
                .split_once("_")
                .map(|(nation, _)| Vessel {
                    id: id.to_owned(),
                    nation: nation.to_owned(),
                })
        });

    Ok(vessels.collect())
}

#[test]
fn test_path() {
    let path = Path::new("/some/file.txt");
    assert_eq!(path.file_stem().unwrap(), "file");

    let stem = path.file_stem().unwrap();
    assert_eq!(stem, "file");
    assert!(stem.to_str().unwrap().ends_with("le"));
}

fn main() {
    let mission_path = dir::mission_dir().join("Random Mission.ini");
    let mut mission = load_template()
        .expect("failed to read mission_template.ini");

    let mut rng = thread_rng();
    let vessels = load_vessels().expect("failed to load vessels");
    mission.set("Mission", "NumberOfNeutralVessels", Some(vessels.len().to_string()));
    for (i, vessel) in vessels.iter().enumerate() {
        if vessel.nation != "civ" {
            continue;
        }

        println!("vessel: {:?}", vessel);

        let section = format!("NeutralVessel{}", i + 1);
        mission.set(&section, "type", Some(vessel.id.clone()));
        // mission.set(&section, "WeaponStatus", Some("Tight".to_owned()));
        mission.set(&section, "CrewSkill", Some("Trained".to_owned()));
        mission.set(&section, "Stores", Some("Full".to_owned()));

        let position: (i32, i32, i32) =
            (rng.gen_range(-10..=10), 0, rng.gen_range(-10..=10));
        let position_str = format!("{},{},{}", position.0, position.1, position.2);
        mission.set(&section, "RelativePositionInNM", Some(position_str));

        let heading: u16 = rng.gen_range(0..360);
        mission.set(&section, "Heading", Some(heading.to_string()));
    }

    write_template(&mission_path, mission)
        .expect("failed to write mission file");
}
