use std::sync::Arc;

use super::UnitOrRandom;
use cursive::align::HAlign;
use cursive::wrap_impl;
use cursive::{view::ViewWrapper, Cursive};
use cursive_table_view::{TableView, TableViewItem};
use cursive_tree_view::{Placement, TreeView};

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

    pub fn borrow_item(&self, row: usize) -> Option<&UnitOrRandom> {
        self.view.borrow_item(row)
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

pub struct UnitTree {
    last_formation_id: usize,
    view: TreeView<UnitTreeItem>,
}

impl UnitTree {
    pub fn new() -> Self {
        Self {
            last_formation_id: 0,
            view: TreeView::new(),
        }
    }

    pub fn on_remove<F>(mut self, cb: F) -> Self
    where
        F: Fn(&mut Cursive, usize) + Send + Sync + 'static,
    {
        let cb = Arc::new(cb);
        self.view.set_on_submit({
            let cb = cb.clone();
            move |s, row| cb(s, row)
        });
        self.view.set_on_collapse({
            let cb = cb.clone();
            move |s, row, _, _| cb(s, row)
        });
        self
    }

    pub fn add_unit(&mut self, unit: UnitOrRandom) {
        let insert_at = self.view.row().unwrap_or(0);
        let placement = self
            .view
            .borrow_item(insert_at)
            .and_then(|item| {
                if let UnitTreeItem::Formation(_) = item {
                    Some(Placement::LastChild)
                } else {
                    None
                }
            })
            .unwrap_or(Placement::After);
        let n = self
            .view
            .insert_item(UnitTreeItem::Unit(unit), placement, insert_at)
            .unwrap_or(0);
        // select newly inserted row
        self.view.set_selected_row(n);
    }

    pub fn add_formation(&mut self) {
        let formation_id = self.next_formation_id();
        let insert_at = self
            .view
            .row()
            .and_then(|row| self.view.item_parent(row).or(Some(row)))
            .unwrap_or(0);
        let n = self
            .view
            .insert_item(
                UnitTreeItem::Formation(formation_id),
                Placement::After,
                insert_at,
            )
            .unwrap_or(0);
        self.view.set_selected_row(n);
    }

    pub fn remove(&mut self, row: usize) {
        // FIXME: there's a bug in cursive_tree_view that if you attempt
        // to delete the last remaining element (with row = 0) it will panic
        // with: attempt to subtract with overflow
        // stack backtrace:
        // 3: cursive_tree_view::TreeView<enum2$<cursive_demo::UnitTreeItem> >::remove_item<enum2$<cursive_demo::UnitTreeItem> >
        //   at C:<REDACTED>\registry\src\index.crates.io-6f17d22bba15001f\cursive_tree_view-0.9.0\src\lib.rs:396
        if self.view.len() > 1 {
            self.view.remove_item(row);
        } else {
            self.view.clear();
        }
    }

    // FIXME: return a different, more useful type
    pub fn selected(&self) -> Vec<UnitTreeItem> {
        // TreeView currently has no way to return a reference to all items, except
        // for take_items (which is not what we want as it will clear the list)
        let mut items: Vec<UnitTreeItem> = Vec::new();
        for row in 0..self.view.len() {
            if let Some(item) = self.view.borrow_item(row) {
                items.push(item.clone());
            }
        }
        items
    }

    fn next_formation_id(&mut self) -> usize {
        self.last_formation_id += 1;
        self.last_formation_id
    }
}

impl ViewWrapper for UnitTree {
    wrap_impl!(self.view: TreeView<UnitTreeItem>);
}
