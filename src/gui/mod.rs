mod views;

use std::sync::{Arc, Mutex};

use crate::mission::MissionOptions;
use crate::mission::{self, UnitOption};
use crate::unit_db as db;

use cursive::reexports::log::{info, LevelFilter};
use cursive::traits::*;
use cursive::views::{
    Button, Dialog, DummyView, EditView, LinearLayout, ListView, Panel, ResizedView, SelectView,
    TextView,
};
use cursive::Cursive;
use cursive_tree_view::Placement;

use views::{selected_units, unit_tree, UnitTable, UnitTree, UnitTreeItem};

const NATIONS: [&'static str; 5] = ["Civilian", "USSR", "China", "Iran", "USA"];

#[derive(Clone, Debug)]
pub struct Unit {
    id: String,
    name: String,
    nation: String,
    // TODO: have a type of unit (e.g. Carrier Unit)
    utype: db::UnitType,
}

#[derive(Clone, Debug)]
pub enum UnitOrRandom {
    Unit(Unit),
    Random {
        nation: Option<String>,
        utype: Option<db::UnitType>,
    },
}

impl UnitOrRandom {
    fn name(&self) -> String {
        match self {
            UnitOrRandom::Unit(unit) => unit.name.clone(),
            UnitOrRandom::Random { nation, utype } => {
                // TODO: cleanup, will want to add more filters later
                match (nation, utype) {
                    (Some(nation), Some(utype)) => format!("<RANDOM {nation} {utype}>"),
                    (Some(nation), None) => format!("<RANDOM {nation}>"),
                    (None, Some(utype)) => format!("<RANDOM {utype}>"),
                    (None, None) => "<RANDOM>".into(),
                }
            }
        }
    }

    fn nation(&self) -> String {
        match self {
            UnitOrRandom::Unit(unit) => unit.nation.clone(),
            UnitOrRandom::Random { nation, .. } => nation.clone().unwrap_or("<RANDOM>".into()),
        }
    }

    fn utype(&self) -> String {
        match self {
            UnitOrRandom::Unit(unit) => unit.utype.to_string(),
            UnitOrRandom::Random { utype, .. } => {
                utype.map_or("RANDOM".into(), |utype| utype.to_string())
            }
        }
    }
}

// #[derive(Clone, Debug)]
// struct MaybeUnit(UnitOrRandom);

fn units() -> Vec<UnitOrRandom> {
    use db::UnitType::*;
    vec![
        UnitOrRandom::Random {
            nation: None,
            utype: None,
        },
        UnitOrRandom::Random {
            nation: Some("USSR".into()),
            utype: None,
        },
        UnitOrRandom::Random {
            nation: Some("USSR".into()),
            utype: Some(Ship),
        },
        UnitOrRandom::Random {
            nation: Some("USSR".into()),
            utype: Some(Submarine),
        },
        UnitOrRandom::Random {
            nation: Some("USA".into()),
            utype: None,
        },
        UnitOrRandom::Random {
            nation: Some("USA".into()),
            utype: Some(Ship),
        },
        UnitOrRandom::Random {
            nation: Some("USA".into()),
            utype: Some(Submarine),
        },
        UnitOrRandom::Random {
            nation: Some("China".into()),
            utype: None,
        },
        UnitOrRandom::Random {
            nation: Some("China".into()),
            utype: Some(Ship),
        },
        UnitOrRandom::Random {
            nation: Some("China".into()),
            utype: Some(Submarine),
        },
        // Civilian
        UnitOrRandom::Unit(Unit {
            id: "civ_ms_act_1".into(),
            name: "ACT 1-class".into(),
            nation: "Civilian".into(),
            utype: Ship,
        }),
        UnitOrRandom::Unit(Unit {
            id: "civ_fv_sampan".into(),
            name: "Sampan".into(),
            nation: "Civilian".into(),
            utype: Ship,
        }),
        UnitOrRandom::Unit(Unit {
            id: "civ_fv_okean".into(),
            name: "Okean-class Trawler".into(),
            nation: "Civilian".into(),
            utype: Ship,
        }),
        UnitOrRandom::Unit(Unit {
            id: "civ_ms_kommunist".into(),
            name: "Kommunist-class".into(),
            nation: "Civilian".into(),
            utype: Ship,
        }),
        // USSR
        UnitOrRandom::Unit(Unit {
            id: "wp_bpk_kresta2".into(),
            name: "Kresta II-class".into(),
            nation: "USSR".into(),
            utype: Ship,
        }),
        UnitOrRandom::Unit(Unit {
            id: "wp_bpk_udaloy".into(),
            name: "Udaloy-class".into(),
            nation: "USSR".into(),
            utype: Ship,
        }),
        UnitOrRandom::Unit(Unit {
            id: "wp_pkr_moskva".into(),
            name: "Moskva-class".into(),
            nation: "USSR".into(),
            utype: Ship,
        }),
        UnitOrRandom::Unit(Unit {
            id: "wp_rkr_kirov".into(),
            name: "Kirov-class".into(),
            nation: "USSR".into(),
            utype: Ship,
        }),
        UnitOrRandom::Unit(Unit {
            id: "wp_ss_kilo".into(),
            name: "Kilo-class".into(),
            nation: "USSR".into(),
            utype: Submarine,
        }),
        // USA
        UnitOrRandom::Unit(Unit {
            id: "usn_ff_knox".into(),
            name: "Knox-class".into(),
            nation: "USA".into(),
            utype: Ship,
        }),
        UnitOrRandom::Unit(Unit {
            id: "usn_ff_garcia".into(),
            name: "Garcia-class".into(),
            nation: "USA".into(),
            utype: Ship,
        }),
        UnitOrRandom::Unit(Unit {
            id: "usn_cv_kitty_hawk".into(),
            name: "Kitty Hawk-class".into(),
            nation: "USA".into(),
            utype: Ship,
        }),
        UnitOrRandom::Unit(Unit {
            id: "usn_cg_leahy".into(),
            name: "Leahy-class".into(),
            nation: "USA".into(),
            utype: Ship,
        }),
    ]
}

pub fn start() {
    cursive::logger::init();
    // turn off internal cursive logging
    cursive::logger::set_internal_filter_level(LevelFilter::Off);

    let mission = Arc::new(Mutex::new(MissionOptions::default()));

    let mut siv = cursive::default();
    siv.set_window_title("Sea Power Mission Generator");
    siv.add_global_callback('q', Cursive::quit);
    siv.add_global_callback('`', Cursive::toggle_debug_console);

    let general_form = ListView::new()
        .child(
            "Latitude/Longitude",
            LinearLayout::horizontal()
                .child(EditView::new().with_name("latitude").fixed_width(6))
                .child(TextView::new(","))
                .child(EditView::new().with_name("longitude").fixed_width(6)),
        )
        .child(
            "Width/Height (nm)",
            LinearLayout::horizontal()
                .child(EditView::new().with_name("size_w").fixed_width(6))
                .child(TextView::new(","))
                .child(EditView::new().with_name("size_h").fixed_width(6)),
        );

    let neutral_form = {
        let mission = mission.clone();
        ListView::new().child(
            "Unit Groups",
            Button::new("Customise...", move |s| {
                let mission = mission.clone();
                let view = customise_group_view(units(), move |s, _| {
                    let mut mission = mission.lock().unwrap();
                    mission.neutral.units = vec![UnitOption::Unit("civ_ms_kommunist".into())];
                    s.pop_layer();
                });
                s.add_layer(view);
            }),
        )
    };

    let blue_form = ListView::new()
        .child(
            "Nation",
            SelectView::new()
                .popup()
                .item_str("USSR")
                .item_str("China")
                .item_str("Iran"),
        )
        .child(
            "Unit Groups",
            Button::new("Customise...", |s| {
                let view = customise_group_view(units(), |s, _| {
                    s.pop_layer();
                });
                s.add_layer(view);
            }),
        );

    let red_form = ListView::new()
        .child(
            "Nation",
            SelectView::new()
                .popup()
                .item_str("USA")
                .item_str("Iraq")
                .item_str("Norway"),
        )
        .child(
            "Unit Groups",
            Button::new("Customise...", |s| {
                let view = customise_group_view(units(), |s, _| {
                    s.pop_layer();
                });
                s.add_layer(view);
            }),
        );

    siv.add_layer(
        Dialog::new()
            .title("Create Mission")
            .button("Generate", {
                let mission = mission.clone();
                move |s| {
                    let mission = mission.lock().unwrap();
                    generate_mission(s, mission.clone());
                }
            })
            .button("Quit", Cursive::quit)
            .content(
                LinearLayout::vertical()
                    .child(Panel::new(general_form).title("General"))
                    .child(Panel::new(neutral_form).title("Neutral"))
                    .child(Panel::new(blue_form).title("Blue"))
                    .child(Panel::new(red_form).title("Red")),
            ),
    );

    siv.run();
}

fn generate_mission(s: &mut Cursive, mut mission: MissionOptions) {
    let lat = s
        .call_on_name("latitude", |view: &mut EditView| view.get_content())
        .unwrap();
    let lon = s
        .call_on_name("longitude", |view: &mut EditView| view.get_content())
        .unwrap();
    // fixme: unwrap
    let latlon = (lat.parse().unwrap(), lon.parse().unwrap());

    let width = s
        .call_on_name("size_w", |view: &mut EditView| view.get_content())
        .unwrap();
    let height = s
        .call_on_name("size_h", |view: &mut EditView| view.get_content())
        .unwrap();
    // fixme: unwrap
    let size = (width.parse().unwrap(), height.parse().unwrap());

    mission.general = mission::GeneralOptions { latlon, size };

    info!("{:?}", mission);
}

fn customise_group_view<F>(available: Vec<UnitOrRandom>, on_submit: F) -> impl View
where
    F: Fn(&mut Cursive, Vec<views::UnitTreeItem>) + Send + Sync + 'static,
{
    fn add_selected(s: &mut Cursive, index: usize) {
        let available = s.find_name::<UnitTable>("available").unwrap();
        if let Some(item) = available.borrow_item(index) {
            s.call_on_name("selected", |selected: &mut UnitTree| {
                let insert_at = selected.row().unwrap_or(0);
                let placement = selected
                    .borrow_item(insert_at)
                    .and_then(|item| {
                        if let UnitTreeItem::Formation(_) = item {
                            Some(Placement::LastChild)
                        } else {
                            None
                        }
                    })
                    .unwrap_or(Placement::After);
                let n = selected
                    .insert_item(UnitTreeItem::Unit(item.clone()), placement, insert_at)
                    .unwrap_or(0);
                // select newly inserted row
                selected.set_selected_row(n);
            });
        }
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

    fn add_formation(s: &mut Cursive, formation_id: Arc<Mutex<usize>>) {
        let mut formation_id = formation_id.lock().unwrap();
        *formation_id += 1;

        s.call_on_name("selected", |selected: &mut UnitTree| {
            let insert_at = selected
                .row()
                .and_then(|row| selected.item_parent(row).or(Some(row)))
                .unwrap_or(0);
            let n = selected
                .insert_item(
                    UnitTreeItem::Formation(*formation_id),
                    Placement::After,
                    insert_at,
                )
                .unwrap_or(0);
            selected.set_selected_row(n);
        });
    }

    fn filter(s: &mut Cursive, _item: &String) {
        let nation = s
            .find_name::<SelectView>("filter_nation")
            .expect("missing filter_nation view")
            .selection()
            // FIXME
            .unwrap();
        let utype = s
            .find_name::<SelectView>("filter_utype")
            .expect("missing filter_utype view")
            .selection()
            .unwrap();

        s.call_on_name("available", |available: &mut UnitTable| {
            available.filter(&nation, &utype);
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
                    .max_width(20),
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
                    .max_width(20),
            )
            .child(
                "Random",
                SelectView::new()
                    .popup()
                    .item_str("<ALL>")
                    .item_str("Only RANDOM")
                    .item_str("No RANDOM")
                    .max_width(20),
            ),
    )
    .title("Filters");

    let available_panel = Panel::new(
        UnitTable::new(available)
            .on_submit(add_selected)
            .with_name("available"),
    )
    .title("Available");

    let formation_id = Arc::new(Mutex::new(0));
    let create_formation_button =
        LinearLayout::horizontal().child(Button::new("Create Formation", move |s| {
            add_formation(s, formation_id.clone());
        }));

    let selected_panel = Panel::new(
        unit_tree()
            .on_submit(remove_selected)
            .on_collapse(|s, row, _, _| {
                remove_selected(s, row);
            })
            .with_name("selected")
            .scrollable(),
    )
    .title("Selected");

    Dialog::new()
        .title("Customise Group")
        .button("Ok", move |s| {
            let selected = selected_units(s);
            on_submit(s, selected);
        })
        .content(
            LinearLayout::vertical()
                .child(filter_panel)
                .child(available_panel.min_size((32, 20)))
                .child(create_formation_button)
                // spacing
                .child(ResizedView::with_fixed_size((4, 0), DummyView))
                .child(selected_panel.min_size((32, 20))),
        )
        .scrollable()
        .full_screen()
}
