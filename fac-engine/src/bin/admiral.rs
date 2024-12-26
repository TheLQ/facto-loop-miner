use std::rc::Rc;

use exhaustive::Exhaustive;
use facto_loop_miner_common::log_init;
use facto_loop_miner_fac_engine::blueprint::bpfac::infinity::{FacBpFilter, FacBpInfinitySettings};
use facto_loop_miner_fac_engine::blueprint::bpitem::BlueprintItem;
use facto_loop_miner_fac_engine::blueprint::output::FacItemOutput;
use facto_loop_miner_fac_engine::common::names::FacEntityName;
use facto_loop_miner_fac_engine::common::vpoint::VPOINT_ZERO;
use facto_loop_miner_fac_engine::game_blocks::belt_bettel::FacBlkBettelBelt;
use facto_loop_miner_fac_engine::game_blocks::rail_hope::RailHopeAppender;
use facto_loop_miner_fac_engine::game_blocks::rail_hope_dual::RailHopeDual;
use facto_loop_miner_fac_engine::game_blocks::rail_hope_single::RailHopeSingle;
use facto_loop_miner_fac_engine::game_blocks::rail_loop::{FacBlkRailLoop, FacBlkRailLoopProps};
use facto_loop_miner_fac_engine::game_blocks::rail_station::FacBlkRailStation;
use facto_loop_miner_fac_engine::game_entities::direction::FacDirectionQuarter;
use facto_loop_miner_fac_engine::game_entities::infinity_power::FacEntInfinityPower;
use facto_loop_miner_fac_engine::game_entities::module::FacModule;
use facto_loop_miner_fac_engine::{
    admiral::{
        err::{AdmiralResult, pretty_panic_admiral},
        executor::client::AdmiralClient,
        lua_command::{LuaCommand, fac_destroy::FacDestroy},
    },
    common::{entity::FacEntity, vpoint::VPoint},
    game_blocks::{assembler_thru::FacBlkAssemblerThru, block::FacBlock},
    game_entities::{
        assembler::FacEntAssembler,
        belt::FacEntBeltType,
        chest::{FacEntChest, FacEntChestType},
        inserter::FacEntInserterType,
        tier::FacTier,
    },
};
use tracing::Level;

fn main() {
    log_init(Some(Level::DEBUG));
    // log_init(Some(Level::TRACE));

    if let Err(e) = inner_main() {
        let msg = pretty_panic_admiral(e);
        panic!("⛔⛔⛔ DEAD: {}", msg)
    }
}

fn inner_main() -> AdmiralResult<()> {
    let mut client = AdmiralClient::new()?;
    client.auth()?;

    let output = FacItemOutput::new_admiral(client).into_rc();

    match 9 {
        1 => make_basic(output)?,
        2 => make_assembler_thru(output)?,
        3 => make_belt_bettel(output)?,
        4 => make_rail_spiral_90(output)?,
        5 => make_rail_shift_45(output)?,
        6 => make_rail_dual_turning(output)?,
        7 => make_rail_dual_powered(output)?,
        8 => make_rail_station(output)?,
        9 => make_rail_loop(output)?,
        _ => panic!("uihhh"),
    }

    Ok(())
}

fn make_basic(output: Rc<FacItemOutput>) -> AdmiralResult<()> {
    execute_destroy(output.clone())?;

    output.write(BlueprintItem::new(
        FacEntChest::new(FacEntChestType::Active).into_boxed(),
        VPoint::zero(),
    ));

    Ok(())
}

fn make_assembler_thru(output: Rc<FacItemOutput>) -> AdmiralResult<()> {
    execute_destroy(output.clone())?;

    let farm = FacBlkAssemblerThru {
        assembler: FacEntAssembler::new(FacTier::Tier3, "copper-cable".into(), [
            Some(FacModule::Speed(FacTier::Tier3)),
            Some(FacModule::Speed(FacTier::Tier3)),
            Some(FacModule::Speed(FacTier::Tier3)),
        ]),
        belt_type: FacEntBeltType::Fast,
        inserter_type: FacEntInserterType::Fast,
        width: 4,
        height: 3,
        output: output.clone(),
    };
    farm.generate(VPoint::new(5, 5));

    Ok(())
}

fn make_belt_bettel(output: Rc<FacItemOutput>) -> AdmiralResult<()> {
    execute_destroy(output.clone())?;

    let mut belt = FacBlkBettelBelt::new(
        FacEntBeltType::Basic,
        VPoint::new(5, 5),
        FacDirectionQuarter::South,
        output.clone(),
    );
    belt.add_straight(5);
    belt.add_turn90(false);
    belt.add_straight_underground(5);
    belt.add_turn90(true);
    belt.add_straight(5);

    Ok(())
}

fn make_rail_spiral_90(output: Rc<FacItemOutput>) -> AdmiralResult<()> {
    execute_destroy(output.clone())?;

    let origin: VPoint = VPoint::zero();
    for clockwise in [
        true,  //
        false, //
    ] {
        let hope1 = RailHopeSingle::new(origin, FacDirectionQuarter::North, output.clone());
        let hope2 = RailHopeSingle::new(origin, FacDirectionQuarter::South, output.clone());
        let hope3 = RailHopeSingle::new(origin, FacDirectionQuarter::East, output.clone());
        let hope4 = RailHopeSingle::new(origin, FacDirectionQuarter::West, output.clone());
        for mut hope in [
            hope1, //
            hope2, //
            hope3, //
            hope4, //
        ] {
            hope.add_straight(2);
            hope.add_turn90(clockwise);
            hope.add_straight(2);
            hope.add_turn90(clockwise);
            hope.add_straight(2);
        }
    }

    Ok(())
}

fn make_rail_shift_45(output: Rc<FacItemOutput>) -> AdmiralResult<()> {
    execute_destroy(output.clone())?;

    let origin = VPoint::zero();
    for clockwise in [false /*, false*/] {
        let hope1 = RailHopeSingle::new(origin, FacDirectionQuarter::North, output.clone());
        let hope2 = RailHopeSingle::new(origin, FacDirectionQuarter::South, output.clone());
        let hope3 = RailHopeSingle::new(origin, FacDirectionQuarter::East, output.clone());
        let hope4 = RailHopeSingle::new(origin, FacDirectionQuarter::West, output.clone());
        for mut hope in [hope1, hope2, hope3, hope4] {
            hope.add_straight(2);
            hope.add_shift45(clockwise, 3);
            hope.add_straight(2)
        }
    }

    Ok(())
}

fn make_rail_dual_turning(output: Rc<FacItemOutput>) -> AdmiralResult<()> {
    execute_destroy(output.clone())?;

    for clockwise in [true, false] {
        for direction in [
            FacDirectionQuarter::North,
            FacDirectionQuarter::East,
            FacDirectionQuarter::South,
            FacDirectionQuarter::West,
        ] {
            let mut hope = RailHopeDual::new(VPoint::zero(), direction, output.clone());
            hope.add_straight(5);
            hope.add_turn90(clockwise);
            hope.add_straight(5);
            hope.add_straight(5);
        }
    }

    Ok(())
}

fn make_rail_dual_powered(output: Rc<FacItemOutput>) -> AdmiralResult<()> {
    execute_destroy(output.clone())?;

    for direction in [
        FacDirectionQuarter::North,
        // FacDirectionQuarter::East,
        // FacDirectionQuarter::South,
        // FacDirectionQuarter::West,
    ] {
        let origin = VPoint::zero().move_direction(&direction, 6);

        let mut hope = RailHopeDual::new(origin, direction, output.clone());
        hope.add_straight_section();
        hope.add_turn90(true);
        hope.add_straight_section();
        hope.add_turn90(true);
        hope.add_straight_section();
        // for entity in hope.to_fac() {
        //     let bpfac = entity.to_blueprint();
        //     // let bppos = &bpfac.position;
        //     // if existing_points.contains(bppos) {
        //     //     continue;
        //     // } else {
        //     //     existing_points.push(bppos.clone());
        //     // }
        //     admiral.execute_checked_command(bpfac.to_lua().into_boxed())?;
        // }
    }

    Ok(())
}

fn make_rail_station(output: Rc<FacItemOutput>) -> AdmiralResult<()> {
    execute_destroy(output.clone())?;

    let station = FacBlkRailStation {
        name: "test".into(),
        wagons: 4,
        front_engines: 0,
        // chests: Some(FacEntChestType::Steel),
        chests: None,
        inserter: FacEntInserterType::Basic,
        is_east: true,
        // is_east: false,
        is_up: true,
        // is_up: false,
        is_input: true,
        is_create_train: true,
        output,
    };
    station.generate(VPOINT_ZERO);
    Ok(())
}

fn make_rail_loop(output: Rc<FacItemOutput>) -> AdmiralResult<()> {
    execute_destroy(output.clone())?;

    let origin = VPoint::zero();

    output.write(BlueprintItem::new(
        FacEntInfinityPower::new().into_boxed(),
        origin.move_xy(4, 2),
    ));
    let mut rail_loop = FacBlkRailLoop::new(FacBlkRailLoopProps {
        name_prefix: "Basic".into(),
        wagons: 3,
        front_engines: 2,
        origin,
        origin_direction: FacDirectionQuarter::West,
        chest_input: Some(FacEntChestType::Infinity(FacBpInfinitySettings {
            remove_unfiltered_items: false,
            filters: vec![FacBpFilter {
                count: 22,
                mode: "at-least".into(),
                name: "iron-stick".into(),
            }],
        })),
        chest_output: Some(FacEntChestType::Infinity(FacBpInfinitySettings {
            remove_unfiltered_items: true,
            filters: vec![
                FacBpFilter {
                    count: 22,
                    mode: "at-least".into(),
                    name: "iron-stick".into(),
                },
                FacBpFilter {
                    count: 22,
                    mode: "at-least".into(),
                    name: "iron-ore".into(),
                },
            ],
        })),
        inserter_type: FacEntInserterType::Stack,
        is_start_input: true,
        output: output.clone(),
    });
    rail_loop.add_turn90(false);
    rail_loop.add_straight();
    rail_loop.add_turn90(false);
    rail_loop.add_base_start_and_end();

    Ok(())
}

fn execute_destroy(output: Rc<FacItemOutput>) -> AdmiralResult<()> {
    let command = FacDestroy::new_filtered(
        150,
        FacEntityName::iter_exhaustive(None)
            .map(|v| v.to_fac_name())
            .collect(),
    );
    // Do not use, this deletes mine resource tiles
    // let command = FacDestroy::new_everything(50);
    output.admiral_execute_command(command.into_boxed())?;

    Ok(())
}
