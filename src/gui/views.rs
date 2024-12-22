use cursive::Cursive;
use cursive::align::HAlign;
use cursive_table_view::{TableView, TableViewItem};
use cursive_tree_view::TreeView;
use super::UnitOrRandom;

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

pub type UnitTable = TableView<UnitOrRandom, UnitColumn>;

pub fn unit_table() -> UnitTable {
    TableView::<UnitOrRandom, UnitColumn>::new()
        .column(UnitColumn::Name, "Name", |c| c.align(HAlign::Left))
        .column(UnitColumn::Nation, "Nation", |c| {
            c.align(HAlign::Center).width_percent(20)
        })
        .column(UnitColumn::Type, "Type", |c| {
            c.align(HAlign::Right).width_percent(20)
        })
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
