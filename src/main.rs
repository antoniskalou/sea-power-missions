mod dir;

use rand::{rngs::ThreadRng, seq::SliceRandom, thread_rng, Rng};
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

#[derive(Debug)]
enum GenOption {
    MinMax(u16, u16),
    Fixed(u16),
}

impl GenOption {
    fn gen(&self, rng: &mut ThreadRng) -> u16 {
        use GenOption::*;
        match *self {
            MinMax(min, max) => rng.gen_range(min..=max),
            Fixed(val) => val,
        }
    }
}

#[derive(Debug)]
struct MissionOptions {
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

    fn gen_position(&self) -> (f32, f32) {
        let mut rng = thread_rng();
        let (w, h) = self.options.size;
        let half_w = w as f32 / 2.0;
        let half_h = h as f32 / 2.0;
        (
            rng.gen_range(-half_w..=half_w),
            rng.gen_range(-half_h..=half_h),
        )
    }
}

fn main() {
    let mission_path = dir::mission_dir().join("Random Mission.ini");
    let mut config = load_template()
        .expect("failed to read mission_template.ini");

    let mission = Mission::new(
        MissionOptions {
            size: (200, 200),
            n_neutral: GenOption::MinMax(10, 30),
            n_blue: GenOption::Fixed(1),
            n_red: GenOption::Fixed(2),
        }
    );
    println!("config: {:?}", mission);

    let mut rng = thread_rng();
    let vessels = load_vessels().expect("failed to load vessels");

    let n_neutral = mission.options.n_neutral.gen(&mut rng);
    let neutrals = vessels
        .iter()
        .filter(|v| v.nation == "civ")
        .collect::<Vec<_>>();
    config.set("Mission", "NumberOfNeutralVessels", Some(n_neutral.to_string()));

    println!("number of neutrals: {}", n_neutral);
    for i in 0..n_neutral {
        let vessel = neutrals.choose(&mut rng).unwrap();
        println!("adding vessel: {:?}", vessel);

        let section = format!("NeutralVessel{}", i + 1);
        config.set(&section, "type", Some(vessel.id.clone()));
        // a.k.a. speed
        config.set(&section, "Telegraph", Some(2.to_string()));
        config.set(&section, "CrewSkill", Some("Trained".to_owned()));
        config.set(&section, "Stores", Some("Full".to_owned()));

        let position = mission.gen_position();
        let position_str = format!("{},0,{}", position.0, position.1);
        config.set(&section, "RelativePositionInNM", Some(position_str));

        let heading: u16 = rng.gen_range(0..360);
        config.set(&section, "Heading", Some(heading.to_string()));
    }

    let n_red = mission.options.n_red.gen(&mut rng);
    let reds = vessels
        .iter()
        .filter(|v| v.nation == "usn")
        .collect::<Vec<_>>();
    config.set("Mission", "NumberOfTaskforce2Vessels", Some(n_red.to_string()));

    println!("number of reds: {}", n_red);
    for i in 0..n_red {
        let vessel = reds.choose(&mut rng).unwrap();
        println!("adding vessel: {:?}", vessel);

        let section = format!("Taskforce2Vessel{}", i + 1);
        config.set(&section, "type", Some(vessel.id.clone()));
        config.set(&section, "Telegraph", Some(2.to_string()));
        config.set(&section, "WeaponStatus", Some("Free".to_owned()));
        config.set(&section, "CrewSkill", Some("Trained".to_owned()));
        config.set(&section, "Stores", Some("Full".to_owned()));

        let position = mission.gen_position();
        let position_str = format!("{},0,{}", position.0, position.1);
        config.set(&section, "RelativePositionInNM", Some(position_str));

        let heading: u16 = rng.gen_range(0..360);
        config.set(&section, "Heading", Some(heading.to_string()));
    }

    write_template(&mission_path, config)
        .expect("failed to write mission file");
}
