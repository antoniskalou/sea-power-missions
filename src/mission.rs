use std::collections::HashMap;

use crate::unit_db::{UnitId, UnitType};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum WeaponState {
    Free,
    Tight,
    Hold,
}

#[derive(Debug)]
pub struct Unit {
    id: String,
    heading: u16,
    position: (f32, f32),
}

#[derive(Clone, Debug)]
pub enum TaskforceUnit {
    Unit(UnitId),
    Random {
        nation: Option<String>,
        subtype: Option<UnitType>,
    },
}

#[derive(Clone, Debug)]
pub struct TaskforceFormation {
    units: Vec<TaskforceUnit>,
}

#[derive(Clone, Debug)]
pub struct TaskforceOptions {
    weapon_state: WeaponState,
    units: Vec<TaskforceUnit>,
    formations: Vec<TaskforceFormation>,
}

type UnitReference = (UnitType, usize);

#[derive(Debug)]
pub struct Taskforce {
    options: TaskforceOptions,
    name: String,
    units: HashMap<UnitType, Vec<Unit>>,
    formations: Vec<Vec<UnitReference>>,
}

impl Taskforce {
    pub fn new(name: &str, options: TaskforceOptions) -> Self {
        Self {
            options,
            name: name.to_owned(),
            units: HashMap::new(),
            formations: vec![],
        }
    }
}

#[derive(Debug)]
struct MissionOptions {
    /// the map center latitude and logitude
    latlon: (f32, f32),
    /// the size of the box (w,h) that the mission will take place in.
    size: (u16, u16),
    neutral: TaskforceOptions,
    blue: TaskforceOptions,
    red: TaskforceOptions,
}

#[derive(Debug)]
struct Mission {
    options: MissionOptions,
    neutral: Taskforce,
    blue: Taskforce,
    red: Taskforce,
}

impl Mission {
    pub fn new(options: MissionOptions) -> Self {
        let neutral = Taskforce::new("Neutral", options.neutral.clone());
        let blue = Taskforce::new("Taskforce1", options.blue.clone());
        let red = Taskforce::new("Taskforce2", options.red.clone());
        Self { options, neutral, blue, red }
    }
}

#[test]
fn test_mission_new() {
    let mission = Mission::new(MissionOptions {
        latlon: (12., 12.),
        size: (100, 100),
        neutral: TaskforceOptions {
            weapon_state: WeaponState::Hold,
            units: vec![
                TaskforceUnit::Unit("civ_ms_kommunist".to_owned()),
            ],
            formations: vec![],
        },
        blue: TaskforceOptions {
            weapon_state: WeaponState::Tight,
            units: vec![
                TaskforceUnit::Unit("wp_rkr_kirov".to_owned()),
            ],
            formations: vec![
                TaskforceFormation {
                    units: vec![
                        TaskforceUnit::Unit("wp_rkr_kirov".to_owned()),
                    ]
                }
            ],
        },
        red: TaskforceOptions {
            weapon_state: WeaponState::Free,
            units: vec![
                TaskforceUnit::Random { nation: Some("usn".to_owned()), subtype: None }
            ],
            formations: vec![],
        }
    });
    println!("{:?}", mission);
}
