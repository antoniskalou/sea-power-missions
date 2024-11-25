use crate::unit_db::{UnitDB, Unit, UnitId, UnitType};
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

pub fn gen_units<'a>(
    unit_db: &'a UnitDB,
    options: &'a Vec<UnitOption>
) -> Vec<&'a Unit> {
    let mut rng = thread_rng();

    let mut units = vec![];
    for option in options {
        match option {
            UnitOption::Unit(id) => {
                if let Some(unit) = unit_db.by_id(&id) {
                    units.push(unit);
                }
            },
            UnitOption::Random { nation, subtype } => {
                let matches = unit_db.search(nation.as_deref(), *subtype);
                if let Some(unit) = matches.choose(&mut rng) {
                    units.push(unit);
                }
            },
            UnitOption::Formation(options) => {
                // println!("\tformation: {options:?}");
                // FIXME: currently will ignore formations during config
                // generation, this needs to be kept around for later
                let mut new_units = gen_units(&unit_db, options);
                units.append(&mut new_units);
                // units.append(gen_units(unit_db, options));
            }
        }
    }
    units
}
