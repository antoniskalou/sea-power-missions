mod reusable_id;
mod views;

use std::sync::{Arc, Mutex};
use itertools::iproduct;

use crate::mission::{self, MissionOptions, TaskforceOptions, UnitOption};
use crate::unit_db::{Nation, Unit, UnitDb, UnitType};

use cursive::reexports::log::{info, LevelFilter};
use cursive::traits::*;
use cursive::views::{
    Button, Dialog, DummyView, EditView, LinearLayout, ListView, Panel, ResizedView, SelectView,
    TextView,
};
use cursive::Cursive;

use views::{UnitTable, UnitTree};

#[derive(Clone, Debug)]
pub enum UnitOrRandom {
    Unit(Unit),
    Random {
        nation: Option<String>,
        utype: Option<UnitType>,
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
            UnitOrRandom::Unit(unit) => unit.nation.name.clone(),
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

impl From<UnitOrRandom> for UnitOption {
    fn from(value: UnitOrRandom) -> Self {
        match value {
            UnitOrRandom::Unit(unit) => UnitOption::Unit(unit.id),
            UnitOrRandom::Random { nation, utype } => UnitOption::Random { nation, utype },
        }
    }
}

// #[derive(Clone, Debug)]
// struct MaybeUnit(UnitOrRandom);

struct AppState {
    all_units: Vec<Unit>,
    nations: Vec<Nation>,
}

pub struct App {
    state: AppState,
}

impl App {
    pub fn new(unit_db: &UnitDb) -> Self {
        let state = AppState {
            all_units: unit_db.all().into_iter().cloned().collect(),
            nations: unit_db.nations().into_iter().cloned().collect(),
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
    let available = Arc::new(convert_units(&state.all_units, &state.nations));
    let nations = Arc::new(state.nations);
    let mission = Arc::new(Mutex::new(MissionOptions::default()));

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
        let available = available.clone();
        let nations = nations.clone();
        let mission = mission.clone();
        ListView::new().child(
            "Unit Groups",
            Button::new("Customise...", move |s| {
                let view = customise_group_view(
                    &available.clone(),
                    &nations.clone(),
                    fill_taskforce(mission.clone(), |mission| &mut mission.neutral),
                );
                s.add_layer(view);
            }),
        )
    };

    let blue_form = ListView::new()
        .child(
            "Nation",
            SelectView::new()
                .popup()
                // FIXME: remove civilian from selection
                .with_all_str(nations.iter())
        )
        .child(
            "Unit Groups",
            Button::new("Customise...", {
                let available = available.clone();
                let nations = nations.clone();
                let mission = mission.clone();
                move |s| {
                    let view = customise_group_view(
                        &available.clone(),
                        &nations.clone(),
                        fill_taskforce(mission.clone(), |mission| &mut mission.blue),
                    );
                    s.add_layer(view);
                }
            }),
        );

    let red_form = ListView::new()
        .child(
            "Nation",
            SelectView::new()
                .popup()
                .with_all_str(nations.iter())
        )
        .child(
            "Unit Groups",
            Button::new("Customise...", {
                let available = available.clone();
                let nations = nations.clone();
                let mission = mission.clone();
                move |s| {
                    let view = customise_group_view(
                        &available.clone(),
                        &nations.clone(),
                        fill_taskforce(mission.clone(), |mission| &mut mission.red),
                    );
                    s.add_layer(view);
                }
            }),
        );

    Dialog::new()
        .title("Create Mission")
        .button("Generate", {
            let mission = mission.clone();
            move |s| {
                let mut mission = mission.lock().unwrap();
                fill_mission(s, &mut mission);
                on_submit(mission.clone());
                // TODO: show more useful info
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
    available: &[UnitOrRandom],
    nations: &Vec<Nation>,
    on_submit: F
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
                selected.add_unit(item.clone());
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
                    .with_all_str(nations)
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
        UnitTable::new(available.to_owned())
            .on_submit(add_selected)
            .with_name("available"),
    )
    .title("Available");

    let selected_panel = Panel::new(
        UnitTree::new()
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
                        .child(Button::new("Create Formation", add_formation)),
                )
                // spacing
                .child(ResizedView::with_fixed_size((4, 0), DummyView))
                .child(selected_panel.min_size((32, 20))),
        )
        .scrollable()
        .full_screen()
}

fn randoms(nations: &[Nation]) -> Vec<UnitOrRandom> {
    let types = UnitType::all();
    iproduct!(
        nations.iter().map(Some).chain(std::iter::once(None)),
        types.iter().map(Some).chain(std::iter::once(None))
    )
    .map(|(nation, utype)| UnitOrRandom::Random {
        nation: nation.cloned().map(|n| n.name.into()),
        utype: utype.copied(),
    })
    .collect::<Vec<_>>()
}

fn convert_units(units: &[Unit], nations: &[Nation]) -> Vec<UnitOrRandom> {
    let mut all_units = randoms(nations);
    all_units.extend(units.iter().map(|unit| UnitOrRandom::Unit(unit.clone())));
    all_units
}

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
