use super::UnitOrRandom;
use cursive::align::HAlign;
use cursive::wrap_impl;
use cursive::{view::ViewWrapper, Cursive};
use cursive_table_view::{TableView, TableViewItem};
use cursive_tree_view::TreeView;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum UnitColumn {
    Name,
    Nation,
    Type,
}

impl TableViewItem<UnitColumn> for UnitOrRandom {
    fn to_column(&self, column: UnitColumn) -> String {
        match column {
            UnitColumn::Name => self.name(),
            UnitColumn::Nation => self.nation(),
            UnitColumn::Type => self.utype(),
        }
    }

    fn cmp(&self, other: &Self, column: UnitColumn) -> std::cmp::Ordering
    where
        Self: Sized,
    {
        match column {
            UnitColumn::Name => self.name().cmp(&other.name()),
            UnitColumn::Nation => self.nation().cmp(&other.nation()),
            UnitColumn::Type => self.nation().cmp(&other.utype()),
        }
    }
}

pub struct UnitTable {
    all_units: Vec<UnitOrRandom>,
    view: TableView<UnitOrRandom, UnitColumn>,
}

impl UnitTable {
    pub fn new(all_units: Vec<UnitOrRandom>) -> Self {
        let view = TableView::<UnitOrRandom, UnitColumn>::new()
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

    pub fn filter(&mut self, nation: &str, utype: &str) {
        self.view.set_items(
            self.all_units
                .iter()
                .filter(|unit| nation == "<ALL>" || nation == unit.nation())
                .filter(|unit| utype == "<ALL>" || utype == unit.utype())
                .cloned()
                .collect(),
        );
    }

    pub fn borrow_item(&self, index: usize) -> Option<&UnitOrRandom> {
        self.view.borrow_item(index)
    }

    pub fn on_submit<F>(mut self, cb: F) -> Self
    where
        F: Fn(&mut Cursive, usize) + Send + Sync + 'static,
    {
        self.view.set_on_submit(move |s, _row, index| cb(s, index));
        self
    }
}

impl ViewWrapper for UnitTable {
    wrap_impl!(self.view: TableView<UnitOrRandom, UnitColumn>);
}

#[derive(Clone, Debug)]
pub enum UnitTreeItem {
    Unit(UnitOrRandom),
    Formation(usize),
}

impl std::fmt::Display for UnitTreeItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use UnitTreeItem::*;
        match self {
            Unit(unit) => write!(f, "{}", unit.name()),
            Formation(id) => write!(f, "Formation {id}"),
        }
    }
}

pub type UnitTree = TreeView<UnitTreeItem>;

pub fn unit_tree() -> UnitTree {
    UnitTree::new()
}

pub fn selected_units(s: &mut Cursive) -> Vec<UnitTreeItem> {
    let selected_view = s
        .find_name::<UnitTree>("selected")
        .expect("selected view missing");
    // TreeView currently has no way to return a reference to all items, except
    // for take_items (which is not what we want as it will clear the list)
    let mut items: Vec<UnitTreeItem> = Vec::new();
    for row in 0..selected_view.len() {
        if let Some(item) = selected_view.borrow_item(row) {
            items.push(item.clone());
        }
    }
    items
}
