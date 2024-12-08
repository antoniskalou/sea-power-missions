use cursive::align::HAlign;
use cursive::reexports::log::{log, Level, LevelFilter};
use cursive::Cursive;
use cursive::views::{Button, Dialog, DummyView, EditView, LinearLayout, ListView, Panel, ResizedView, SelectView, TextView};
use cursive::traits::*;
use cursive_table_view::{TableView, TableViewItem};
use cursive_tree_view::{Placement, TreeView};

const NATIONS: [&'static str; 5] = [
    "Civilian",
    "USSR",
    "China",
    "Iran",
    "USA",
];


#[derive(Clone, Debug)]
struct Unit {
    id: String,
    name: String,
    nation: String,
    // TODO: have a type of unit (e.g. Carrier Unit)
    utype: String,
}

#[derive(Clone, Debug)]
enum UnitOption {
    Unit(Unit),
    Random {
        nation: Option<String>,
        utype: Option<String>,
    }
}

impl UnitOption {
    fn name(&self) -> String {
        match self {
            UnitOption::Unit(unit) => unit.name.clone(),
            UnitOption::Random { nation, utype } => {
                // TODO: cleanup, will want to add more filters later
                match (nation, utype) {
                    (Some(nation), Some(utype)) => format!("<RANDOM {nation} {utype}>"),
                    (Some(nation), None) => format!("<RANDOM {nation}>"),
                    (None, Some(utype)) => format!("<RANDOM {utype}>"),
                    (None, None) => "<RANDOM>".to_owned(),
                }
            },
        }
    }

    fn nation(&self) -> String {
        match self {
            UnitOption::Unit(unit) => unit.nation.clone(),
            UnitOption::Random { nation, .. } =>
                nation.clone().unwrap_or("<RANDOM>".to_owned()),
        }
    }

    fn utype(&self) -> String {
        match self {
            UnitOption::Unit(unit) => unit.utype.clone(),
            UnitOption::Random { utype, .. } =>
                utype.clone().unwrap_or("<RANDOM>".to_owned()),
        }
    }
}

// #[derive(Clone, Debug)]
// struct MaybeUnit(UnitOption);

fn units() -> Vec<UnitOption> {
    vec![
        UnitOption::Random {
            nation: None,
            utype: None,
        },
        UnitOption::Random {
            nation: Some("USSR".to_owned()),
            utype: None,
        },
        UnitOption::Random {
            nation: Some("USSR".to_owned()),
            utype: Some("Ship".to_owned()),
        },
        UnitOption::Random {
            nation: Some("USSR".to_owned()),
            utype: Some("Submarine".to_owned()),
        },
        UnitOption::Random {
            nation: Some("USA".to_owned()),
            utype: None,
        },
        UnitOption::Random {
            nation: Some("USA".to_owned()),
            utype: Some("Ship".to_owned()),
        },
        UnitOption::Random {
            nation: Some("USA".to_owned()),
            utype: Some("Submarine".to_owned()),
        },
        UnitOption::Random {
            nation: Some("China".to_owned()),
            utype: None,
        },
        UnitOption::Random {
            nation: Some("China".to_owned()),
            utype: Some("Ship".to_owned()),
        },
        UnitOption::Random {
            nation: Some("China".to_owned()),
            utype: Some("Submarine".to_owned()),
        },
        // Civilian
        UnitOption::Unit(Unit {
            id: "civ_ms_act_1".to_owned(),
            name: "ACT 1-class".to_owned(),
            nation: "Civilian".to_owned(),
            utype: "Ship".to_owned(),
        }),
        UnitOption::Unit(Unit {
            id: "civ_fv_sampan".to_owned(),
            name: "Sampan".to_owned(),
            nation: "Civilian".to_owned(),
            utype: "Ship".to_owned(),
        }),
        UnitOption::Unit(Unit {
            id: "civ_fv_okean".to_owned(),
            name: "Okean-class Trawler".to_owned(),
            nation: "Civilian".to_owned(),
            utype: "Ship".to_owned(),
        }),
        UnitOption::Unit(Unit {
            id: "civ_ms_kommunist".to_owned(),
            name: "Kommunist-class".to_owned(),
            nation: "Civilian".to_owned(),
            utype: "Ship".to_owned(),
        }),
        // USSR
        UnitOption::Unit(Unit {
            id: "wp_bpk_kresta2".to_owned(),
            name: "Kresta II-class".to_owned(),
            nation: "USSR".to_owned(),
            utype: "Ship".to_owned(),
        }),
        UnitOption::Unit(Unit {
            id: "wp_bpk_udaloy".to_owned(),
            name: "Udaloy-class".to_owned(),
            nation: "USSR".to_owned(),
            utype: "Ship".to_owned(),
        }),
        UnitOption::Unit(Unit {
            id: "wp_pkr_moskva".to_owned(),
            name: "Moskva-class".to_owned(),
            nation: "USSR".to_owned(),
            utype: "Ship".to_owned(),
        }),
        UnitOption::Unit(Unit {
            id: "wp_rkr_kirov".to_owned(),
            name: "Kirov-class".to_owned(),
            nation: "USSR".to_owned(),
            utype: "Ship".to_owned(),
        }),
        UnitOption::Unit(Unit {
            id: "wp_ss_kilo".to_owned(),
            name: "Kilo-class".to_owned(),
            nation: "USSR".to_owned(),
            utype: "Submarine".to_owned(),
        }),
        // USA
        UnitOption::Unit(Unit {
            id: "usn_ff_knox".to_owned(),
            name: "Knox-class".to_owned(),
            nation: "USA".to_owned(),
            utype: "Ship".to_owned(),
        }),
        UnitOption::Unit(Unit {
            id: "usn_ff_garcia".to_owned(),
            name: "Garcia-class".to_owned(),
            nation: "USA".to_owned(),
            utype: "Ship".to_owned(),
        }),
        UnitOption::Unit(Unit {
            id: "usn_cv_kitty_hawk".to_owned(),
            name: "Kitty Hawk-class".to_owned(),
            nation: "USA".to_owned(),
            utype: "Ship".to_owned(),
        }),
        UnitOption::Unit(Unit {
            id: "usn_cg_leahy".to_owned(),
            name: "Leahy-class".to_owned(),
            nation: "USA".to_owned(),
            utype: "Ship".to_owned(),
        }),
    ]
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
enum UnitColumn {
    Name,
    Nation,
    Type,
}

impl TableViewItem<UnitColumn> for UnitOption {
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

type UnitTable = TableView<UnitOption, UnitColumn>;
// TODO: TreeView<Unit> with Unit implementing Display

#[derive(Debug)]
enum UnitTreeItem {
    Unit(UnitOption),
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

type UnitTree = TreeView<UnitTreeItem>;

#[derive(Debug)]
struct UnitGroupState {
    last_formation_id: Option<usize>,
}

impl UnitGroupState {
    fn new() -> Self {
        UnitGroupState {
            last_formation_id: None,
        }
    }

    fn formation_id(&mut self) -> usize {
        let new_id = self.last_formation_id
            .map(|id| id + 1)
            .unwrap_or(1);
        self.last_formation_id = Some(new_id);
        new_id
    }
}

pub fn start() {
    cursive::logger::init();
    // turn off internal cursive logging
    cursive::logger::set_internal_filter_level(LevelFilter::Off);

    let mut siv = cursive::default();
    siv.set_window_title("Sea Power Mission Generator");
    siv.add_global_callback('q', Cursive::quit);
    siv.add_global_callback('`', Cursive::toggle_debug_console);

    let general_form = ListView::new()
        .child(
            "Latitude/Longitude",
            LinearLayout::horizontal()
                .child(
                    EditView::new()
                        .with_name("latitude")
                        .fixed_width(6),
                )
                .child(TextView::new(","))
                .child(
                    EditView::new()
                        .with_name("longitude")
                        .fixed_width(6)
                )
        )
        .child(
            "Width/Height (nm)",
            LinearLayout::horizontal()
                .child(
                    EditView::new()
                        .with_name("size_w")
                        .fixed_width(6)
                )
                .child(TextView::new(","))
                .child(
                    EditView::new()
                        .with_name("size_h")
                        .fixed_width(6)
                )
        );

    let neutral_form = ListView::new()
        .child(
            "Unit Groups",
            Button::new("Customise...", |s| {
                customise_group(s, units());
            })
        );

    let blue_form = ListView::new()
        .child(
            "Nation",
            SelectView::new()
                .popup()
                .item_str("USSR")
                .item_str("China")
                .item_str("Iran")
        )
        .child(
            "Unit Groups",
            Button::new("Customise...", |s| {
                customise_group(s, units());
            })
        );

    let red_form = ListView::new()
        .child(
            "Nation",
            SelectView::new()
                .popup()
                .item_str("USA")
                .item_str("Iraq")
                .item_str("Norway")
        )
        .child(
            "Unit Groups",
            Button::new("Customise...", |s| {
                customise_group(s, units());
            })
        );

    siv.add_layer(
        Dialog::new()
            .title("Create Mission")
            .button("Generate", Cursive::quit)
            .button("Quit", Cursive::quit)
            .content(
                LinearLayout::vertical()
                    .child(Panel::new(general_form).title("General"))
                    .child(Panel::new(neutral_form).title("Neutral"))
                    .child(Panel::new(blue_form).title("Blue"))
                    .child(Panel::new(red_form).title("Red"))
            )
    );

    siv.run();
}

fn customise_group(s: &mut Cursive, available: Vec<UnitOption>) {
    fn add_selected(s: &mut Cursive, _row: usize, index: usize) {
        let available = s.find_name::<UnitTable>("available").unwrap();
        s.call_on_name("selected", |selected: &mut UnitTree| {
            if let Some(item) = available.borrow_item(index) {
                let insert_at = selected.row().unwrap_or(0);
                let placement = selected.borrow_item(insert_at)
                    .and_then(|item| {
                        if let UnitTreeItem::Formation(_) = item {
                            Some(Placement::LastChild)
                        } else {
                            None
                        }
                    })
                    .unwrap_or(Placement::After);
                let n = selected.insert_item(
                    UnitTreeItem::Unit(item.clone()),
                    placement,
                    insert_at
                ).unwrap_or(0);
                // select newly inserted row
                selected.set_selected_row(n);
            }
        });
    }

    fn remove_selected(s: &mut Cursive, row: usize) {
        s.call_on_name("selected", |selected: &mut UnitTree| {
            // FIXME: there's a bug in cursive_tree_view that if you attempt
            // to delete the last remaining element (with row = 0) it will panic
            // with: attempt to subtract with overflow
            // stack backtrace:
            // 3: cursive_tree_view::TreeView<enum2$<cursive_demo::UnitTreeItem> >::remove_item<enum2$<cursive_demo::UnitTreeItem> >
             //   at C:<REDACTED>\registry\src\index.crates.io-6f17d22bba15001f\cursive_tree_view-0.9.0\src\lib.rs:396
            if selected.len() > 1 {
                selected.remove_item(row);
            } else {
                selected.clear();
            }
        });
    }

    fn add_formation(s: &mut Cursive) {
        let formation_id =
            s.with_user_data(|user_data: &mut UnitGroupState| {
                user_data.formation_id()
            }).expect("user data not set");

        s.call_on_name("selected", |selected: &mut UnitTree| {
            let insert_at = selected.row()
                .and_then(|row| selected.item_parent(row).or(Some(row)))
                .unwrap_or(0);
            let n = selected.insert_item(
                UnitTreeItem::Formation(formation_id),
                Placement::After,
                insert_at
            ).unwrap_or(0);
            selected.set_selected_row(n);
        });
    }

    fn filter(s: &mut Cursive, _item: &String) {
        let nation = s.find_name::<SelectView>("filter_nation")
            // FIXME
            .unwrap().selection().unwrap();
        let utype = s.find_name::<SelectView>("filter_utype")
            .unwrap().selection().unwrap();
        let units = units();

        s.call_on_name("available", |available: &mut UnitTable| {
            available.set_items(
                units.iter()
                    .filter(|unit| {
                        *nation == "<ALL>" || *nation == unit.nation()
                    })
                    .filter(|unit| {
                        *utype == "<ALL>" || *utype == unit.utype()
                    })
                    .cloned()
                    .collect()
            );
        });
    }

    let filter_panel = Panel::new(
        ListView::new()
            .child(
                "Nation",
                SelectView::new()
                    .popup()
                    .item_str("<ALL>")
                    .with_all_str(NATIONS)
                    .on_submit(filter)
                    .with_name("filter_nation")
                    .max_width(20)
            )
            .child(
                "Type",
                SelectView::new()
                    .popup()
                    .item_str("<ALL>")
                    .item_str("Ship")
                    .item_str("Submarine")
                    .on_submit(filter)
                    .with_name("filter_utype")
                    .max_width(20)
            )
            .child(
                "Random",
                SelectView::new()
                    .popup()
                    .item_str("<ALL>")
                    .item_str("Only RANDOM")
                    .item_str("No RANDOM")
                    .max_width(20)
            )
    ).title("Filters");

    let available_panel = Panel::new(
        unit_table()
            .items(available)
            .on_submit(add_selected)
            .with_name("available")
    ).title("Available");

    let create_formation_button =
           LinearLayout::horizontal()
            // TODO: figure out spacing
            .child(Button::new("Create Formation", add_formation));

    let selected_panel = Panel::new(
        UnitTree::new()
            .on_submit(remove_selected)
            .on_collapse(|s, row, _, _| { remove_selected(s, row); })
            .with_name("selected")
            .scrollable()
    ).title("Selected");

    s.set_user_data(UnitGroupState::new());
    s.add_layer(
        Dialog::new()
            .title("Customise Group")
            .button("Ok", |s| { s.pop_layer(); })
            .button("Cancel", |s| { s.pop_layer(); })
            .content(
                LinearLayout::vertical()
                    .child(filter_panel)
                    .child(available_panel.min_size((32, 20)))
                    .child(create_formation_button)
                    // spacing
                    .child(ResizedView::with_fixed_size((4, 0), DummyView))
                    .child(selected_panel.min_size((32, 20)))
            )
            .scrollable()
            .full_screen()

    );
}

fn unit_table() -> UnitTable {
    TableView::<UnitOption, UnitColumn>::new()
        .column(UnitColumn::Name, "Name", |c| c.align(HAlign::Left))
        .column(UnitColumn::Nation, "Nation", |c| {
            c.align(HAlign::Center)
                .width_percent(20)
        })
        .column(UnitColumn::Type, "Type", |c| {
            c
                .align(HAlign::Right)
                .width_percent(20)
        })
}
