use crate::unit_db::{UnitDB, Vessel, VesselType};
use rand::{rngs::ThreadRng, seq::IteratorRandom, thread_rng, Rng};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum GenOption {
    MinMax(u16, u16),
    Fixed(u16),
}

impl GenOption {
    pub fn gen(&self, rng: &mut ThreadRng) -> u16 {
        use GenOption::*;
        match *self {
            MinMax(min, max) => rng.gen_range(min..=max),
            Fixed(val) => val,
        }
    }
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

pub fn gen_neutrals<'a>(n: &'a GenOption, unit_db: &'a UnitDB) -> Vec<&'a Vessel> {
    let mut rng = thread_rng();
    let n = n.gen(&mut rng);
    unit_db
        .vessels_by_nation("civ")
        .iter()
        .map(|v| *v)
        .choose_multiple(&mut rng, n as usize)
}

pub fn gen_blues<'a>(n: &'a GenOption, unit_db: &'a UnitDB) -> Vec<&'a Vessel> {
    let mut rng = thread_rng();
    let n = n.gen(&mut rng);
    unit_db
        .vessels_by_nation("wp")
        .iter()
        // only use ships for now, subs aren't ready yet
        .filter(|v| v.subtype == VesselType::Ship)
        .map(|v| *v)
        .choose_multiple(&mut rng, n as usize)
}

pub fn gen_reds<'a>(n: &'a GenOption, unit_db: &'a UnitDB) -> Vec<&'a Vessel> {
    let mut rng = thread_rng();
    let n = n.gen(&mut rng);
    unit_db
        .vessels_by_nation("usn")
        .iter()
        // only use ships for now, subs aren't ready yet
        .filter(|v| v.subtype == VesselType::Ship)
        .map(|v| *v)
        .choose_multiple(&mut rng, n as usize)
}
