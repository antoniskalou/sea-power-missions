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
    pub fn filter(&mut self, nation: Option<&str>, utype: Option<&str>) {
        self.view.set_items(
            self.all_units
                .iter()
                .filter(|unit| nation.map(|n| n == unit.nation.to_string()).unwrap_or(true))
                .filter(|unit| utype.map(|t| t == unit.utype.to_string()).unwrap_or(true))
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

    fn cmp(&self, other: &Self, column: UnitColumn) -> std::cmp::Ordering {
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

        unit1.utype = db::UnitType::Aircraft;
        unit2.utype = db::UnitType::Aircraft;
        assert_eq!(unit1.cmp(&unit2, UnitColumn::Type), Ordering::Equal);

        unit2.utype = db::UnitType::Vessel;
        assert_eq!(unit1.cmp(&unit2, UnitColumn::Type), Ordering::Less);
        assert_eq!(unit2.cmp(&unit1, UnitColumn::Type), Ordering::Greater);
    }

    fn assert_eq_table(table: &UnitTable, row: usize, unit: &db::Unit) {
        assert_eq!(
            table.borrow_item(row).map(|u| u.id.clone()),
            Some(unit.id.clone())
        );
    }

    #[test]
    fn unit_table_borrow_item() {
        let units: Vec<db::Unit> = (0..3).map(|_| fake_unit()).collect();
        let table = UnitTable::new(units.clone());
        // order should be preserved
        for (row, unit) in units.iter().enumerate() {
            assert_eq_table(&table, row, unit);
        }
        // nothing to borrow past end of list
        assert_eq!(table.borrow_item(units.len()).map(|u| u.id.clone()), None);
    }

    #[test]
    fn unit_table_filter() {
        let units: Vec<db::Unit> = (0..3).map(|_| fake_unit()).collect();
        let mut table = UnitTable::new(units.clone());

        let nation = &units[0].nation.to_string();
        let utype = &units[0].utype.to_string();

        // table should remain the same
        table.filter(None, None);
        assert!(table.borrow_item(units.len() - 1).is_some());

        // only one nation
        table.filter(Some(nation), None);
        for (row, unit) in units.iter().enumerate() {
            if unit.nation.to_string() == *nation {
                assert_eq_table(&table, row, &unit);
            }
        }

        // only one type
        table.filter(None, Some(utype));
        for (row, unit) in units.iter().enumerate() {
            if unit.utype.to_string() == *utype {
                assert_eq_table(&table, row, &unit);
            }
        }

        // both nation and type
        table.filter(Some(nation), Some(utype));
        for (row, unit) in units.iter().enumerate() {
            if unit.nation.to_string() == *nation && unit.utype.to_string() == *utype {
                assert_eq_table(&table, row, &unit);
            }
        }

        // neither
        table.filter(Some("MISSING"), Some("MISSING"));
        assert!(table.borrow_item(0).is_none());
    }
}
