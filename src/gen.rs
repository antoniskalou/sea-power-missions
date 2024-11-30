use crate::taskforce::Taskforce;
use crate::unit_db::{UnitDb, Unit, UnitId, UnitType};
use rand::{seq::SliceRandom, thread_rng, Rng};

#[derive(Clone, Debug)]
pub enum UnitOption {
    Unit(UnitId),
    Random {
        nation: Option<String>,
        subtype: Option<UnitType>,
    },
    // FIXME: not actually recursive, since a nested
    // formation is not allowed
    Formation(Vec<UnitOption>),
}

pub fn gen_taskforce(
    taskforce: &mut Taskforce,
    unit_db: &UnitDb,
    options: &Vec<UnitOption>,
) {
    for unit in gen_units(unit_db, options) {
        match unit {
            UnitOrFormation::Unit(unit) => {
                taskforce.add(&unit);
            },
            UnitOrFormation::Formation(formation) => {
                taskforce.add_formation(&formation);
            }
        }
    }
}

type Formation = Vec<Unit>;

#[derive(Clone, Debug)]
enum UnitOrFormation {
    Unit(Unit),
    Formation(Formation),
}

fn gen_units(
    unit_db: &UnitDb,
    options: &Vec<UnitOption>
) -> Vec<UnitOrFormation> {
    let mut rng = thread_rng();

    let mut units = vec![];
    for option in options {
        match option {
            UnitOption::Unit(id) => {
                if let Some(unit) = unit_db.by_id(&id) {
                    units.push(UnitOrFormation::Unit(unit.clone()));
                }
            },
            UnitOption::Random { nation, subtype } => {
                let matches = unit_db.search(nation.as_deref(), *subtype);
                if let Some(unit) = matches.choose(&mut rng) {
                    units.push(UnitOrFormation::Unit((*unit).clone()));
                }
            },
            UnitOption::Formation(options) => {
                let new_units = gen_units(&unit_db, options)
                    .iter()
                    .filter_map(|u| match u {
                        UnitOrFormation::Unit(unit) => Some(unit),
                        UnitOrFormation::Formation(_) => None,
                    })
                    .cloned()
                    .collect();
                units.push(UnitOrFormation::Formation(new_units));
            }
        }
    }
    units
}

#[derive(Debug)]
pub struct Position(f32, f32);

impl Position {
    pub fn new(x: f32, y: f32) -> Self {
        Self(x, y)
    }

    pub fn x(&self) -> f32 {
        self.0
    }

    pub fn y(&self) -> f32 {
        self.1
    }
}

impl ToString for Position {
    fn to_string(&self) -> String {
        format!("{},0,{}", self.x(), self.y())
    }
}

pub fn gen_position(size: &(u16, u16)) -> Position {
    let mut rng = thread_rng();
    let (w, h) = *size;
    let half_w = w as f32 / 2.0;
    let half_h = h as f32 / 2.0;
    Position::new(
        rng.gen_range(-half_w..=half_w),
        rng.gen_range(-half_h..=half_h),
    )
}

pub fn gen_heading() -> u16 {
    thread_rng().gen_range(0..360)
}
