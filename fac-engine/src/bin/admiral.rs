use std::rc::Rc;

use exhaustive::Exhaustive;
use facto_loop_miner_common::log_init_trace;
use facto_loop_miner_fac_engine::admiral::lua_command::train_boot::train_boot;
use facto_loop_miner_fac_engine::blueprint::bpfac::infinity::{FacBpFilter, FacBpInfinitySettings};
use facto_loop_miner_fac_engine::blueprint::bpfac::schedule::{
    FacBpCircuitCondition, FacBpLogic, FacBpSchedule, FacBpScheduleData, FacBpScheduleWait,
    FacBpSignalId, FacBpSignalIdType, FacBpWaitType,
};
use facto_loop_miner_fac_engine::blueprint::bpitem::BlueprintItem;
use facto_loop_miner_fac_engine::blueprint::output::FacItemOutput;
use facto_loop_miner_fac_engine::common::names::FacEntityName;
use facto_loop_miner_fac_engine::common::varea::VArea;
use facto_loop_miner_fac_engine::common::vpoint::{VPOINT_TEN, VPOINT_ZERO};
use facto_loop_miner_fac_engine::game_blocks::belt_bettel::FacBlkBettelBelt;
use facto_loop_miner_fac_engine::game_blocks::belt_train_unload::FacBlkBeltTrainUnload;
use facto_loop_miner_fac_engine::game_blocks::rail_hope::RailHopeAppender;
use facto_loop_miner_fac_engine::game_blocks::rail_hope_dual::RailHopeDual;
use facto_loop_miner_fac_engine::game_blocks::rail_hope_single::RailHopeSingle;
use facto_loop_miner_fac_engine::game_blocks::rail_loop::{FacBlkRailLoop, FacBlkRailLoopProps};
use facto_loop_miner_fac_engine::game_blocks::rail_station::{FacBlkRailStation, FacExtDelivery};
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

fn main() {
    log_init_trace();
    // log_init_debug();

    if let Err(e) = inner_main() {
        let msg = pretty_panic_admiral(e);
        panic!("⛔⛔⛔ DEAD: {}", msg)
    }
}

fn inner_main() -> AdmiralResult<()> {
    let mut client = AdmiralClient::new()?;
    client.auth()?;

    let output = FacItemOutput::new_admiral(client).into_rc();

    match 3 {
        1 => make_basic(output)?,
        2 => make_assembler_thru(output)?,
        3 => make_belt_bettel(output)?,
        4 => make_rail_spiral_90(output)?,
        5 => make_rail_shift_45(output)?,
        6 => make_rail_dual_turning(output)?,
        7 => make_rail_dual_powered(output)?,
        8 => make_rail_station(output)?,
        9 => make_rail_loop(output)?,
        10 => make_belt_bettel_train_unload(output)?,
        _ => panic!("uihhh"),
    }

    Ok(())
}

fn make_basic(output: Rc<FacItemOutput>) -> AdmiralResult<()> {
    execute_destroy(output.clone())?;

    output.write(BlueprintItem::new(
        FacEntChest::new(FacEntChestType::Active).into_boxed(),
        VPOINT_ZERO,
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

    let direction = FacDirectionQuarter::North;

    let mut belt =
        FacBlkBettelBelt::new(FacEntBeltType::Basic, VPOINT_TEN, direction, output.clone());
    // belt.add_straight(5);
    // belt.add_turn90(false);
    // belt.add_straight_underground(5);
    // belt.add_turn90(true);
    belt.add_straight(1);
    belt.add_split(true);
    belt.add_straight(1);

    let mut belt = FacBlkBettelBelt::new(
        FacEntBeltType::Basic,
        VPOINT_TEN.move_x(10),
        direction,
        output.clone(),
    );
    // belt.add_straight(5);
    // belt.add_turn90(false);
    // belt.add_straight_underground(5);
    // belt.add_turn90(true);
    belt.add_straight(5);
    belt.add_split(false);
    belt.add_straight(5);

    Ok(())
}

fn make_belt_bettel_train_unload(output: Rc<FacItemOutput>) -> AdmiralResult<()> {
    execute_destroy(output.clone())?;

    let block = FacBlkBeltTrainUnload {
        belt_type: FacEntBeltType::Basic,
        output: output.clone(),
        wagons: 2,
        padding_unmerged: 0, // 2,
        padding_above: 0,
        padding_after: 0,
        turn_clockwise: true,
        origin_direction: FacDirectionQuarter::East,
    };
    block.generate(VPOINT_ZERO);

    Ok(())
}

fn make_rail_spiral_90(output: Rc<FacItemOutput>) -> AdmiralResult<()> {
    execute_destroy(output.clone())?;

    let origin: VPoint = VPOINT_ZERO;
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

    let origin = VPOINT_ZERO;
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
            let mut hope = RailHopeDual::new(VPOINT_ZERO, direction, output.clone());
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
        let origin = VPOINT_ZERO.move_direction_usz(&direction, 6);

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
        wagons: 2,
        front_engines: 2,
        delivery: FacExtDelivery::Belt(FacEntBeltType::Basic),
        // chests: Some(FacEntChestType::Steel),
        // chests: None,
        inserter: FacEntInserterType::Basic,
        fuel_inserter: Some(FacEntInserterType::Basic),
        fuel_inserter_chest: Some(FacEntChestType::Steel),
        is_east: true,
        // is_east: false,
        is_up: true,
        // is_up: false,
        is_input: true,
        is_create_train: true,
        schedule: Some(FacBpSchedule {
            locomotives: Vec::new(),
            schdata: [
                FacBpScheduleData {
                    station: "SomeTestStart".into(),
                    wait_conditions: [
                        FacBpScheduleWait {
                            compare_type: FacBpLogic::Or,
                            ctype: FacBpWaitType::ItemCount,
                            condition: Some(FacBpCircuitCondition {
                                comparator: Some("<".into()),
                                first_signal: Some(FacBpSignalId {
                                    stype: FacBpSignalIdType::Item,
                                    name: "heavy-oil-barrel".into(),
                                }),
                                second_signal: None,
                                constant: Some(800),
                            }),
                        },
                        FacBpScheduleWait {
                            compare_type: FacBpLogic::Or,
                            ctype: FacBpWaitType::Empty,
                            condition: None,
                        },
                    ]
                    .into(),
                },
                FacBpScheduleData {
                    station: "SomeTestEnd".into(),
                    wait_conditions: [FacBpScheduleWait {
                        compare_type: FacBpLogic::Or,
                        ctype: FacBpWaitType::Full,
                        condition: None,
                    }]
                    .into(),
                },
            ]
            .into(),
        }),
        output,
    };
    station.generate(VPOINT_ZERO);
    Ok(())
}

fn make_rail_loop(output: Rc<FacItemOutput>) -> AdmiralResult<()> {
    execute_destroy(output.clone())?;

    let origin = VPOINT_ZERO;

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
        delivery_input: FacExtDelivery::Chest(FacEntChestType::Infinity(FacBpInfinitySettings {
            remove_unfiltered_items: false,
            filters: vec![FacBpFilter::new_for_item("iron-stick")],
        })),
        delivery_output: FacExtDelivery::Chest(FacEntChestType::Infinity(FacBpInfinitySettings {
            remove_unfiltered_items: true,
            filters: vec![
                FacBpFilter::new_for_item("iron-stick"),
                FacBpFilter::new_for_item("iron-ore"),
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

    output.admiral_execute_command(
        train_boot(VArea::from_arbitrary_points_pair(
            VPoint::new(-90, -90),
            VPoint::new(90, 90),
        ))
        .into_boxed(),
    )?;

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
