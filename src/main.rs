mod dir;

use rand::seq::IteratorRandom;
use rand::{rngs::ThreadRng, seq::SliceRandom, thread_rng, Rng};
use std::{os::windows::thread, path::Path};
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

impl Vessel {
    // TODO: move to Mission::write_vessel
    fn write_config(
        &self,
        config: &mut Ini,
        section: &str,
        position: &(f32, f32),
        heading: u16
    ) {
        config.set(&section, "type", Some(self.id.clone()));
        // speed setting
        config.set(&section, "Telegraph", Some(2.to_string()));
        // defaults to "Green"
        config.set(&section, "CrewSkill", Some("Trained".to_owned()));
        // defaults to "Depleted"
        config.set(&section, "Stores", Some("Full".to_owned()));

        let position_str = format!("{},0,{}", position.0, position.1);
        config.set(&section, "RelativePositionInNM", Some(position_str));
        config.set(&section, "Heading", Some(heading.to_string()));
    }
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

fn gen_neutrals(
    mission: &Mission,
    vessels: &Vec<Vessel>
) -> Vec<Vessel> {
    let mut rng = thread_rng();
    let n = mission.options.n_neutral.gen(&mut rng);
    vessels
        .iter()
        .filter(|v| v.nation == "civ")
        .map(|v| v.clone())
        .choose_multiple(&mut rng, n as usize)
}

fn gen_blues(
    mission: &Mission,
    vessels: &Vec<Vessel>
) -> Vec<Vessel> {
    let mut rng = thread_rng();
    let n = mission.options.n_blue.gen(&mut rng);
    vessels
        .iter()
        .filter(|v| v.nation == "wp")
        .map(|v| v.clone())
        .choose_multiple(&mut rng, n as usize)
}

fn gen_reds(
    mission: &Mission,
    vessels: &Vec<Vessel>
) -> Vec<Vessel> {
    let mut rng = thread_rng();
    let n = mission.options.n_red.gen(&mut rng);
    vessels
        .iter()
        .filter(|v| v.nation == "usn")
        .map(|v| v.clone())
        .choose_multiple(&mut rng, n as usize)
}

fn add_formation<T>(config: &mut Ini, taskforce: &str, vessels: &Vec<T>) {
    let n = vessels.len();
    let sections = (0..n)
        .map(|i| format!("Taskforce1Vessel{}", i + 1))
        .collect::<Vec<_>>();
    let formation = sections.join(",");
    // OverrideSpawnPositions allows us to place our units anywhere and the formation
    // will be adjusted by the game on mission start (which is exactly what we want)
    let formation_str = format!("{}|Group Name 1|Circle|1.5|OverrideSpawnPositions", formation);
    let key = format!("{}_Formation1", taskforce);
    config.set("Mission", &key, Some(formation_str));
}

fn main() {
    let mission_path = dir::mission_dir().join("Random Mission.ini");
    let mut config = load_template()
        .expect("failed to read mission_template.ini");

    let mission = Mission::new(
        MissionOptions {
            size: (100, 100),
            n_neutral: GenOption::MinMax(10, 30),
            n_blue: GenOption::Fixed(5),
            n_red: GenOption::Fixed(5),
        }
    );
    println!("config: {:?}", mission);

    let mut rng = thread_rng();
    let vessels = load_vessels().expect("failed to load vessels");

    let neutrals = gen_neutrals(&mission, &vessels);
    let n_neutral = neutrals.len();
    config.set("Mission", "NumberOfNeutralVessels", Some(n_neutral.to_string()));

    println!("number of neutrals: {}", n_neutral);
    for (i, vessel) in neutrals.iter().enumerate() {
        println!("adding neutral: {:?}", vessel);

        let section = format!("NeutralVessel{}", i + 1);
        let position = mission.gen_position();
        let heading: u16 = rng.gen_range(0..360);
        vessel.write_config(&mut config, &section, &position, heading);
    }

    let blues = gen_blues(&mission, &vessels);
    let n_blue = blues.len();
    println!("number of blues: {}", n_blue);
    config.set("Mission", "NumberOfTaskforce1Vessels", Some(n_blue.to_string()));
    // put them into formations for now
    add_formation(&mut config, "Taskforce1", &blues);
    config.set("Mission", "Taskforce1_NumberOfFormations", Some(1.to_string()));
    for (i, vessel) in blues.iter().enumerate() {
        println!("adding blue: {:?}", vessel);

        let section = format!("Taskforce1Vessel{}", i + 1);
        let position = mission.gen_position();
        let heading: u16 = rng.gen_range(0..360);

        vessel.write_config(&mut config, &section, &position, heading);
    }

    let reds = gen_reds(&mission, &vessels);
    let n_red = reds.len();
    config.set("Mission", "NumberOfTaskforce2Vessels", Some(n_red.to_string()));
    add_formation(&mut config, "Taskforce2", &reds);
    config.set("Mission", "Taskforce2_NumberOfFormations", Some(1.to_string()));
    println!("number of reds: {}", n_red);
    for (i, vessel) in reds.iter().enumerate() {
        println!("adding red: {:?}", vessel);

        let section = format!("Taskforce2Vessel{}", i + 1);
        let position = mission.gen_position();
        let heading: u16 = rng.gen_range(0..360);

        vessel.write_config(&mut config, &section, &position, heading);
        config.set(&section, "WeaponStatus", Some("Free".to_owned()));
    }

    write_template(&mission_path, config)
        .expect("failed to write mission file");
}
