use crate::mission::{FormationOption, TaskforceOptions, UnitOption};

use crate::gui::reusable_id::ReusableId;
use cursive::view::ViewWrapper;
use cursive::wrap_impl;
use cursive_tree_view::{Placement, TreeView};

#[derive(Clone, Debug)]
pub struct UnitSelection {
    // a UnitOption::Unit may have a count if we want to allow a user to
    // create e.g. 10x a specific unit. I'd like in the future to have
    // a feature where you can select a unit in the tree and press a keybind
    // to increase or decrease the number vessels.
    unit: UnitOption,
    count: usize,
}

impl UnitSelection {
    fn new(unit: UnitOption) -> Self {
        Self { unit, count: 1 }
    }

    fn name(&self) -> String {
        let unit_str = match &self.unit {
            UnitOption::Unit(unit) => unit.name.clone(),
            UnitOption::Random { nation, utype } => {
                // TODO: cleanup, will want to add more filters later
                match (nation, utype) {
                    (Some(nation), Some(utype)) => format!("<RANDOM {nation} {utype}>"),
                    (Some(nation), None) => format!("<RANDOM {nation}>"),
                    (None, Some(utype)) => format!("<RANDOM {utype}>"),
                    (None, None) => "<RANDOM>".into(),
                }
            }
        };

        if self.count > 1 {
            format!("{unit_str} x {}", self.count)
        } else {
            unit_str
        }
    }

    /// Return the options that are associated with this unit.
    ///
    /// Multiple may be returned, since a unit selection can have multiple
    /// instances of the same unit.
    fn as_options(&self) -> Vec<UnitOption> {
        std::iter::repeat_n(self.unit.clone(), self.count).collect()
    }
}

#[derive(Clone, Debug)]
pub enum UnitTreeItem {
    Unit(UnitSelection),
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

/// All selected items from the `UnitTree`.
// TODO: consider removing
#[derive(Debug)]
pub struct UnitTreeSelection {
    pub units: Vec<UnitSelection>,
    pub formations: Vec<Vec<UnitSelection>>,
}

impl UnitTreeSelection {
    /// Fill taskforce units and formations with the selections.
    pub fn fill_taskforce(&self, taskforce: &mut TaskforceOptions) {
        taskforce.units = self.unit_options();
        taskforce.formations = self.formation_options();
    }

    fn unit_options(&self) -> Vec<UnitOption> {
        units_to_options(&self.units)
    }

    fn formation_options(&self) -> Vec<FormationOption> {
        self.formations
            .iter()
            .map(|f| FormationOption {
                units: units_to_options(f),
            })
            .collect()
    }
}

impl From<&TaskforceOptions> for UnitTreeSelection {
    fn from(taskforce: &TaskforceOptions) -> Self {
        let units = options_to_units(&taskforce.units);
        let formations = taskforce
            .formations
            .iter()
            .map(|f| options_to_units(&f.units))
            .collect();
        Self { units, formations }
    }
}

fn options_to_units(opts: &[UnitOption]) -> Vec<UnitSelection> {
    // TODO: count duplicates
    opts.iter().cloned().map(UnitSelection::new).collect()
}

fn units_to_options(units: &[UnitSelection]) -> Vec<UnitOption> {
    units.iter().flat_map(UnitSelection::as_options).collect()
}

/// A tree view that keeps track of units and associated formations.
pub struct UnitTree {
    formation_id: ReusableId,
    view: TreeView<UnitTreeItem>,
}

impl UnitTree {
    pub fn new() -> Self {
        Self {
            formation_id: ReusableId::default(),
            view: TreeView::new(),
        }
    }

    pub fn with_selection(mut self, selection: UnitTreeSelection) -> Self {
        for unit in selection.units {
            self.add_unit(unit.unit);
        }

        for formation in selection.formations {
            self.add_formation();
            // FIXME: can use recursion for this
            for unit in formation {
                self.add_unit(unit.unit);
            }
        }

        self
    }

    // TODO: allow creating with items so as to re-fill list if re-opened
    // pub fn with_items(items: Vec<UnitSelection>) -> Self {}

    fn add_unit_selection(&mut self, selection: UnitSelection) {
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
            .insert_item(UnitTreeItem::Unit(selection), placement, insert_at)
            .unwrap_or(0);
        // select newly inserted row
        self.view.set_selected_row(n);
    }

    /// Add a unit to the tree, this will either be top level or if part of a
    /// formation if previously defined.
    pub fn add_unit(&mut self, unit: UnitOption) {
        self.add_unit_selection(UnitSelection::new(unit))
    }

    pub fn add_n_units(&mut self, unit: UnitOption, count: usize) {
        self.add_unit_selection(UnitSelection { unit, count })
    }

    /// Add a formation to the tree, any units added after this will be added
    /// under this formation, until another formation has been created.
    pub fn add_formation(&mut self) {
        let formation_id = self.formation_id.next();
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

    /// Remove an item from the list.
    pub fn remove(&mut self, row: usize) {
        if let Some(UnitTreeItem::Formation(id)) = self.view.borrow_item(row) {
            self.formation_id.release(*id);
        }

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

    pub fn row(&self) -> Option<usize> {
        self.view.row()
    }

    /// Return all selected items (units & formations) from the tree.
    pub fn selected(&self) -> UnitTreeSelection {
        let mut units = Vec::new();
        let mut formations: Vec<Vec<UnitSelection>> = Vec::new();
        for item in self.items() {
            match item {
                UnitTreeItem::Unit(unit) => {
                    // if we had previously added a formation then all subsequent
                    // units will be part of that formation.
                    if let Some(formation) = formations.last_mut() {
                        formation.push(unit.clone());
                    } else {
                        units.push(unit.clone());
                    }
                }
                UnitTreeItem::Formation(_) => {
                    formations.push(Vec::new());
                }
            }
        }

        UnitTreeSelection { units, formations }
    }

    fn items(&self) -> impl Iterator<Item = &UnitTreeItem> {
        // TreeView currently has no way to return a reference to all items, except
        // for take_items (which is not what we want as it will clear the list)
        (0..self.view.len()).filter_map(|row| self.view.borrow_item(row))
    }
}

impl ViewWrapper for UnitTree {
    wrap_impl!(self.view: TreeView<UnitTreeItem>);
}
