mod dir;
mod mission;
mod rand_ext;
mod unit_db;

use configparser::ini::Ini;
use mission::{
    FormationOption, GeneralOptions, Mission, MissionOptions, TaskforceOptions, UnitOption,
    WeaponState,
};
use std::error::Error;
use std::str;
use unit_db::UnitDb;

const MISSION_TEMPLATE: &'static str = include_str!("../resources/mission_template.ini");

fn load_template() -> Result<Ini, String> {
    let mut config = Ini::new_cs();
    config.read(MISSION_TEMPLATE.into())?;
    Ok(config)
}

fn main() -> Result<(), Box<dyn Error>> {
    let unit_db = UnitDb::new().expect("failed to initialise UnitBD");
    // println!("{:?}", unit_db);

    let mission = Mission::new(
        &unit_db,
        MissionOptions {
            general: GeneralOptions {
                latlon: (34.31, 29.62),
                size: (100, 100),
            },
            neutral: TaskforceOptions {
                weapon_state: WeaponState::Hold,
                units: vec![
                    UnitOption::Unit("civ_ms_kommunist".into()),
                    UnitOption::Random {
                        nation: Some("civ".into()),
                        utype: None,
                    },
                    UnitOption::Random {
                        nation: Some("civ".into()),
                        utype: None,
                    },
                ],
                formations: vec![],
            },
            blue: TaskforceOptions {
                weapon_state: WeaponState::Tight,
                units: vec![UnitOption::Random {
                    nation: Some("wp".into()),
                    utype: Some(unit_db::UnitType::Submarine),
                }],
                formations: vec![FormationOption {
                    units: vec![
                        // UnitOption::Unit("wp_cv_orel".into()),
                        UnitOption::Unit("wp_rkr_kirov".into()),
                        // UnitOption::Unit("wp_bpk_udaloy".into()),
                        // UnitOption::Unit("wp_em_sovremenny".into()),
                        UnitOption::Unit("wp_bpk_kresta2".into()),
                        UnitOption::Unit("wp_bpk_kashin".into()),
                        // UnitOption::Unit("wp_bpk_kashin".into()),
                    ],
                }],
            },
            red: TaskforceOptions {
                weapon_state: WeaponState::Free,
                units: vec![UnitOption::Random {
                    nation: Some("usn".into()),
                    utype: Some(unit_db::UnitType::Submarine),
                }],
                formations: vec![FormationOption {
                    units: vec![
                        // UnitOption::Unit("usn_cg_ticonderoga".into()),
                        UnitOption::Unit("usn_cgn_virginia".into()),
                        UnitOption::Unit("usn_dd_spruance".into()),
                        // UnitOption::Unit("usn_dd_adams_late".into()),
                        UnitOption::Unit("usn_ff_knox".into()),
                        UnitOption::Unit("usn_ff_garcia".into()),
                        UnitOption::Random {
                            nation: Some("knm".into()),
                            utype: Some(unit_db::UnitType::Ship),
                        },
                    ],
                }],
            },
        },
    );
    println!("{:#?}", mission);

    let mut config = load_template()?;
    mission.write_ini(&mut config);
    println!("========== TEMPLATE ==========\n{}", config.writes());

    let mission_path = dir::mission_dir().join("Random Mission.ini");
    config.write(mission_path)?;

    Ok(())
}
