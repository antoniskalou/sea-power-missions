use crate::unit_db as db;

use cursive::align::HAlign;
use cursive::wrap_impl;
use cursive::{view::ViewWrapper, Cursive};
use cursive_table_view::{TableView, TableViewItem};

/// A table view that keeps track of all available units.
pub struct UnitTable {
    all_units: Vec<db::Unit>,
    view: TableView<db::Unit, UnitColumn>,
}

impl UnitTable {
    /// Create a new unit table with a list of all available units.
    pub fn new(all_units: Vec<db::Unit>) -> Self {
        let view = TableView::<db::Unit, UnitColumn>::new()
            .column(UnitColumn::Name, "Name", |c| c.align(HAlign::Left))
            .column(UnitColumn::Nation, "Nation", |c| {
                c.align(HAlign::Center).width_percent(20)
            })
            .column(UnitColumn::Type, "Type", |c| {
                c.align(HAlign::Right).width_percent(20)
            })
            .items(all_units.clone());
        Self { all_units, view }
    }

    /// Filter units by nation or type.
    pub fn filter(&mut self, nation: &str, utype: &str) {
        self.view.set_items(
            self.all_units
                .iter()
                .filter(|unit| nation == "<ALL>" || nation == unit.nation.name)
                .filter(|unit| utype == "<ALL>" || utype == unit.utype.to_string())
                .cloned()
                .collect(),
        );
    }

    // pub fn search(&mut self, name: &str) { }

    /// Return an immutable reference to the unit at the given row.
    pub fn borrow_item(&self, row: usize) -> Option<&db::Unit> {
        self.view.borrow_item(row)
    }

    /// Callback for when a unit has been "submitted", i.e. has been selected
    /// for addition into another view.
    pub fn on_submit<F>(mut self, cb: F) -> Self
    where
        F: Fn(&mut Cursive, usize) + Send + Sync + 'static,
    {
        self.view.set_on_submit(move |s, _row, index| cb(s, index));
        self
    }
}

impl ViewWrapper for UnitTable {
    wrap_impl!(self.view: TableView<db::Unit, UnitColumn>);
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum UnitColumn {
    Name,
    Nation,
    Type,
}

impl TableViewItem<UnitColumn> for db::Unit {
    fn to_column(&self, column: UnitColumn) -> String {
        match column {
            UnitColumn::Name => self.name.clone(),
            UnitColumn::Nation => self.nation.to_string(),
            UnitColumn::Type => self.utype.to_string(),
        }
    }

    fn cmp(&self, other: &Self, column: UnitColumn) -> std::cmp::Ordering
    where
        Self: Sized,
    {
        match column {
            UnitColumn::Name => self.name.cmp(&other.name),
            UnitColumn::Nation => self.nation.name.cmp(&other.nation.name),
            UnitColumn::Type => self.utype.to_string().cmp(&other.utype.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fake::{Fake, Faker};
    use rand::{seq::SliceRandom, thread_rng};
    use std::cmp::Ordering;

    fn fake_unit() -> db::Unit {
        db::Unit {
            id: Faker.fake(),
            name: Faker.fake(),
            nation: db::Nation {
                id: Faker.fake(),
                name: Faker.fake(),
            },
            utype: *db::UnitType::all().choose(&mut thread_rng()).unwrap(),
        }
    }

    #[test]
    fn db_unit_to_column() {
        let unit = fake_unit();
        assert_eq!(unit.to_column(UnitColumn::Name), unit.name);
        assert_eq!(unit.to_column(UnitColumn::Nation), unit.nation.to_string());
        assert_eq!(unit.to_column(UnitColumn::Type), unit.utype.to_string());
    }

    // TODO: consider using quickcheck to make these tests more extensive
    #[test]
    fn db_unit_cmp_name() {
        let mut unit1 = fake_unit();
        let mut unit2 = fake_unit();

        unit1.name = "a".into();
        unit2.name = "a".into();
        assert_eq!(unit1.cmp(&unit2, UnitColumn::Name), Ordering::Equal);

        unit2.name = "b".into();
        assert_eq!(unit1.cmp(&unit2, UnitColumn::Name), Ordering::Less);
        assert_eq!(unit2.cmp(&unit1, UnitColumn::Name), Ordering::Greater);
    }

    #[test]
    fn db_unit_cmp_nation() {
        let mut unit1 = fake_unit();
        let mut unit2 = fake_unit();

        unit1.nation.name = "a".into();
        unit2.nation.name = "a".into();
        assert_eq!(unit1.cmp(&unit2, UnitColumn::Nation), Ordering::Equal);

        unit2.nation.name = "b".into();
        assert_eq!(unit1.cmp(&unit2, UnitColumn::Nation), Ordering::Less);
        assert_eq!(unit2.cmp(&unit1, UnitColumn::Nation), Ordering::Greater);
    }

    #[test]
    fn db_unit_cmp_type() {
        let mut unit1 = fake_unit();
        let mut unit2 = fake_unit();

        unit1.utype = db::UnitType::FixedWing;
        unit2.utype = db::UnitType::FixedWing;
        assert_eq!(unit1.cmp(&unit2, UnitColumn::Type), Ordering::Equal);

        unit2.utype = db::UnitType::Ship;
        assert_eq!(unit1.cmp(&unit2, UnitColumn::Type), Ordering::Less);
        assert_eq!(unit2.cmp(&unit1, UnitColumn::Type), Ordering::Greater);
    }
}
