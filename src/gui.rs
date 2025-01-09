mod reusable_id;
mod views;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::mission::{self, MissionOptions, TaskforceOptions, UnitOption};
use crate::unit_db::{Nation, Unit, UnitDb, UnitType};

use cursive::reexports::log::LevelFilter;
use cursive::traits::*;
use cursive::views::{
    Button, Dialog, DummyView, EditView, LinearLayout, ListView, Panel, ResizedView, SelectView,
    TextView,
};
use cursive::Cursive;

use views::{DefaultSelectView, UnitTable, UnitTree, UnitTreeSelection};

/// Create a dialog asking the user to insert the game path, die and return
/// the user-given path.
///
/// Intended to be called multiple times if need be.
///
/// `show_error` is set to true if a validation error should be shown to the
/// user. This is useful if an attempt was previously made and failed.
pub fn ask_for_game_path(show_error: bool) -> Option<PathBuf> {
    let mut siv = cursive::default();
    siv.set_window_title("Sea Power Location Picker");

    let root = Arc::new(Mutex::new(None));
    siv.add_layer(
        Dialog::around(
            LinearLayout::vertical()
                .child(TextView::new(
                    "Failed to find your Sea Power install, please paste it below...",
                ))
                .child(EditView::new().with_name("path")),
        )
        .button("Ok", {
            let root = Arc::clone(&root);
            move |s| {
                let content = s.call_on_name("path", |v: &mut EditView| v.get_content());
                *root.lock().unwrap() = content;
                s.quit();
            }
        })
        .title("Game location not found"),
    );

    if show_error {
        siv.add_layer(Dialog::info(
            "Given path was invalid or did not exist, try again...",
        ));
    }

    siv.run();

    let root = root.lock().unwrap();
    root.as_deref().map(PathBuf::from)
}

#[derive(Clone, Debug)]
struct AppState {
    all_units: Arc<Vec<Unit>>,
    nations: Arc<Vec<Nation>>,
    mission: Arc<Mutex<MissionOptions>>,
}

pub struct App {
    state: AppState,
}

impl App {
    pub fn new(unit_db: &UnitDb) -> Self {
        let all_units = unit_db.all().into_iter().cloned().collect();
        let nations = unit_db.nations().into_iter().cloned().collect();
        let state = AppState {
            all_units: Arc::new(all_units),
            nations: Arc::new(nations),
            mission: Arc::new(Mutex::new(MissionOptions::default())),
        };
        Self { state }
    }

    pub fn run<F>(self, on_submit: F)
    where
        F: Fn(MissionOptions) + Send + Sync + 'static,
    {
        cursive::logger::init();
        // turn off internal cursive logging
        cursive::logger::set_internal_filter_level(LevelFilter::Off);

        let mut siv = cursive::default();
        siv.set_window_title("Sea Power Mission Generator");
        siv.add_global_callback('`', Cursive::toggle_debug_console);

        siv.add_layer(main_view(self.state, on_submit));
        siv.run();
    }
}

fn main_view<F>(state: AppState, on_submit: F) -> impl View
where
    F: Fn(MissionOptions) + Send + Sync + 'static,
{
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
        let state = state.clone();
        ListView::new().child(
            "Unit Groups",
            Button::new("Customise...", {
                let state = state.clone();
                move |s| {
                    let mission = state.mission.lock().unwrap();
                    let view = customise_group_view(
                        &state,
                        &mission.neutral,
                        fill_taskforce(state.mission.clone(), |m| &mut m.neutral),
                    );
                    s.add_layer(view);
                }
            }),
        )
    };

    let blue_form = ListView::new()
        .child("Nation", nation_select_view(&state.nations))
        .child(
            "Unit Groups",
            Button::new("Customise...", {
                let state = state.clone();
                move |s| {
                    let mission = state.mission.lock().unwrap();
                    let view = customise_group_view(
                        &state,
                        &mission.blue,
                        fill_taskforce(state.mission.clone(), |m| &mut m.blue),
                    );
                    s.add_layer(view);
                }
            }),
        );

    let red_form = ListView::new()
        .child("Nation", nation_select_view(&state.nations))
        .child(
            "Unit Groups",
            Button::new("Customise...", {
                let state = state.clone();
                move |s| {
                    let mission = state.mission.lock().unwrap();
                    let view = customise_group_view(
                        &state,
                        &mission.red,
                        fill_taskforce(state.mission.clone(), |m| &mut m.red),
                    );
                    s.add_layer(view);
                }
            }),
        );

    Dialog::new()
        .title("Create Mission")
        .button("Generate", {
            move |s| {
                let mut mission = state.mission.lock().unwrap();
                fill_mission(s, &mut mission);
                on_submit(mission.clone());
                // TODO: show info on where it was generated
                s.add_layer(Dialog::info("Mission generated!"));
            }
        })
        .button("Quit", Cursive::quit)
        .content(
            LinearLayout::vertical()
                .child(Panel::new(general_form).title("General"))
                .child(Panel::new(neutral_form).title("Neutral"))
                .child(Panel::new(blue_form).title("Blue"))
                .child(Panel::new(red_form).title("Red")),
        )
}

fn customise_group_view<F>(
    state: &AppState,
    taskforce: &TaskforceOptions,
    on_submit: F,
) -> impl View
where
    F: Fn(&mut Cursive, views::UnitTreeSelection) + Send + Sync + 'static,
{
    fn add_selected(s: &mut Cursive, row: usize) {
        let available = s
            .find_name::<UnitTable>("available")
            .expect("missing available view");
        if let Some(item) = available.borrow_item(row) {
            s.call_on_name("selected", |selected: &mut UnitTree| {
                selected.add_unit(UnitOption::Unit(item.clone()));
            });
        }
    }

    fn remove_selected(s: &mut Cursive, row: usize) {
        s.call_on_name("selected", |selected: &mut UnitTree| {
            selected.remove(row);
        });
    }

    fn add_formation(s: &mut Cursive) {
        s.call_on_name("selected", |selected: &mut UnitTree| {
            selected.add_formation();
        });
    }

    fn add_random(s: &mut Cursive, state: AppState) {
        s.add_layer(random_unit_view(&state, |s, nation, utype, count| {
            s.call_on_name("selected", |selected: &mut UnitTree| {
                let utype = utype.clone().map(|u| {
                    UnitType::try_from(u).expect("invalid utype returned from `selected`")
                });
                selected.add_n_units(
                    UnitOption::Random {
                        nation: nation.clone(),
                        utype,
                    },
                    count,
                )
            });
        }));
    }

    fn filter<T>(s: &mut Cursive, _item: &Option<T>) {
        let nation = s
            .find_name::<DefaultSelectView<Nation>>("filter_nation")
            .expect("missing filter_nation view")
            .selection();
        let utype = s
            .find_name::<DefaultSelectView<UnitType>>("filter_utype")
            .expect("missing filter_utype view")
            .selection();

        s.call_on_name("available", |available: &mut UnitTable| {
            let nation_str = nation.map(|n| n.to_string());
            let utype_str = utype.map(|u| u.to_string());
            available.filter(nation_str.as_deref(), utype_str.as_deref());
        });
    }

    let filter_panel = Panel::new(
        ListView::new()
            .child(
                "Nation",
                DefaultSelectView::new("<ALL>")
                    .popup()
                    .with_all(state.nations.iter().cloned())
                    .on_submit(filter)
                    .with_name("filter_nation")
                    .max_width(20),
            )
            .child(
                "Type",
                DefaultSelectView::new("<ALL>")
                    .popup()
                    .with_all(UnitType::all().into_iter())
                    .on_submit(filter)
                    .with_name("filter_utype")
                    .max_width(20),
            ),
    )
    .title("Filters");

    let available_panel = Panel::new(
        UnitTable::new(state.all_units.to_vec())
            .on_submit(add_selected)
            .with_name("available"),
    )
    .title("Available");

    let selected_panel = Panel::new(
        UnitTree::new()
            .with_selection(UnitTreeSelection::from(taskforce))
            .on_remove(remove_selected)
            .with_name("selected")
            .scrollable(),
    )
    .title("Selected");

    Dialog::new()
        .title("Customise Group")
        .button("Ok", move |s| {
            let view = s
                .find_name::<UnitTree>("selected")
                .expect("missing selected view");
            on_submit(s, view.selected());
        })
        .content(
            LinearLayout::vertical()
                .child(filter_panel)
                .child(available_panel.min_size((32, 20)))
                .child(
                    LinearLayout::horizontal()
                        .child(Button::new("Create Formation", add_formation))
                        .child(Button::new("Create Random", {
                            let state = state.clone();
                            move |s| add_random(s, state.clone())
                        })),
                )
                // spacing
                .child(ResizedView::with_fixed_size((4, 0), DummyView))
                .child(selected_panel.min_size((32, 20))),
        )
        .scrollable()
        .full_screen()
}

fn random_unit_view<F>(state: &AppState, on_submit: F) -> impl View
where
    // FIXME: calls with &str, we want &Nation & &UnitType
    F: Fn(&mut Cursive, &Option<String>, &Option<String>, usize) + Send + Sync + 'static,
{
    Dialog::around(
        ListView::new()
            .child(
                "Nation",
                DefaultSelectView::new("<ANY>")
                    .popup()
                    .with_all(state.nations.iter().cloned())
                    .with_name("random_nation")
                    .max_width(20),
            )
            .child(
                "Type",
                DefaultSelectView::new("<ANY>")
                    .popup()
                    .with_all(UnitType::all().into_iter())
                    .with_name("random_type")
                    .max_width(20),
            )
            .child(
                "Number",
                EditView::new()
                    .content(1.to_string())
                    .max_content_width(3)
                    .with_name("random_count")
                    .fixed_width(4),
            ),
    )
    .button("Create", move |s| {
        let nation = s
            .call_on_name("random_nation", |view: &mut DefaultSelectView<Nation>| {
                view.selection().map(|n| n.to_string())
            })
            .expect("missing random_nation view");
        let utype = s
            .call_on_name("random_type", |view: &mut DefaultSelectView<UnitType>| {
                view.selection().map(|u| u.to_string())
            })
            .expect("missing random_type view");
        let count = s
            .call_on_name("random_count", |view: &mut EditView| {
                // TODO: input validation
                view.get_content().parse().expect("parse of count failed")
            })
            .expect("missing random_count view");
        on_submit(s, &nation, &utype, count);
        s.pop_layer();
    })
    .button("Cancel", |s| {
        s.pop_layer();
    })
}

/// A `SelectView` of all nations, excluding civilian. It is intended to be
/// be used for selecting a nation for red & blue, so civilian doesn't make
/// sense.
fn nation_select_view(nations: &[Nation]) -> SelectView {
    let mut nations: Vec<String> = nations
        .iter()
        .filter(|n| n.id != "civ")
        .map(|n| n.to_string())
        .collect();
    // NOTE: we need to sort before dedup, otherwise dedup won't remove all
    // elements properly.
    nations.sort();
    nations.dedup();
    SelectView::new().popup().with_all_str(nations.iter())
}

// Fill mission options based off what is currently in the UI.
fn fill_mission(s: &mut Cursive, mission: &mut MissionOptions) {
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
}

/// Fill taskforce options based off what was selected by in the UI.
///
/// The `fetcher` returns the correct taskforce given a mission. This is so that
/// we can use it for all taskforce options (red, blue, neutral). Unfortunately,
/// we can't simplify this one much further given that everything has to be
/// Send + Sync.
fn fill_taskforce<F>(
    mission: Arc<Mutex<MissionOptions>>,
    fetcher: F,
) -> impl Fn(&mut Cursive, views::UnitTreeSelection) + Send + Sync
where
    F: Fn(&mut MissionOptions) -> &mut TaskforceOptions + Send + Sync,
{
    move |s, selected: views::UnitTreeSelection| {
        let mut mission = mission.lock().unwrap();
        let taskforce = fetcher(&mut mission);
        selected.fill_taskforce(taskforce);
        s.pop_layer();
    }
}
