mod reusable_id;
mod views;

use std::sync::{Arc, Mutex};

use crate::mission::{self, MissionOptions, TaskforceOptions, UnitOption};
use crate::unit_db::{Nation, Unit, UnitDb, UnitType};

use cursive::reexports::log::{info, LevelFilter};
use cursive::traits::*;
use cursive::views::{
    Button, Dialog, DummyView, EditView, LinearLayout, ListView, Panel, ResizedView, SelectView,
    TextView,
};
use cursive::Cursive;

use views::{UnitTable, UnitTree, UnitTreeSelection};

#[derive(Clone, Debug)]
pub struct UnitSelection(UnitOption);

impl UnitSelection {
    // TODO: move this to UnitTree, since its the only one using it and use
    // UnitOption directly instead.
    fn name(&self) -> String {
        match &self.0 {
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
        }
    }
}

impl From<UnitSelection> for UnitOption {
    fn from(value: UnitSelection) -> Self {
        value.0
    }
}

// #[derive(Clone, Debug)]
// struct MaybeUnit(UnitSelection);

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
        .child(
            "Nation",
            SelectView::new()
                .popup()
                // FIXME: remove civilian from selection
                .with_all_str(state.nations.iter()),
        )
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
        .child(
            "Nation",
            SelectView::new().popup().with_all_str(state.nations.iter()),
        )
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
                selected.add_unit(UnitSelection(UnitOption::Unit(item.clone())));
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
        s.add_layer(random_unit_view(&state, |_| {
            info!("adding a new random item...");
        }));
    }

    fn filter(s: &mut Cursive, _item: &str) {
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
                    .with_all_str(state.nations.iter())
                    .on_submit(filter)
                    .with_name("filter_nation")
                    .max_width(20),
            )
            .child(
                "Type",
                SelectView::new()
                    .popup()
                    .item_str("<ALL>")
                    .item_str("Vessel")
                    .item_str("Submarine")
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
    F: Fn(&mut Cursive) + Send + Sync + 'static,
{
    Dialog::around(
        ListView::new()
            .child(
                "Nation",
                SelectView::new()
                    .popup()
                    .item_str("<ANY>")
                    .with_all_str(state.nations.iter())
                    .with_name("random_nation")
                    .max_width(20),
            )
            .child(
                "Type",
                SelectView::new()
                    .popup()
                    .item_str("<ANY>")
                    .with_all_str(UnitType::all())
                    .with_name("random_type")
                    .max_width(20),
            ),
    )
    .button("Create", move |s| on_submit(s))
    .button("Cancel", |s| {
        s.pop_layer();
    })
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
