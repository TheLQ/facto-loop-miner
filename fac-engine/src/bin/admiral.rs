use exhaustive::Exhaustive;
use facto_loop_miner_common::log_init;
use facto_loop_miner_fac_engine::blueprint::bpfac::position::FacBpPosition;
use facto_loop_miner_fac_engine::blueprint::bpitem::BlueprintItem;
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
        executor::{LuaCompiler, client::AdmiralClient},
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

    if let Err(e) = inner_main() {
        let msg = pretty_panic_admiral(e);
        panic!("DEAD: {}", msg)
    }
}

fn inner_main() -> AdmiralResult<()> {
    let mut client = AdmiralClient::new()?;
    client.auth()?;

    match 9 {
        1 => make_basic(&mut client)?,
        2 => make_assembler_thru(&mut client)?,
        3 => make_belt_bettel(&mut client)?,
        4 => make_rail_spiral_90(&mut client)?,
        5 => make_rail_shift_45(&mut client)?,
        6 => make_rail_dual_turning(&mut client)?,
        7 => make_rail_dual_powered(&mut client)?,
        8 => make_rail_station(&mut client)?,
        9 => make_rail_loop(&mut client)?,
        _ => panic!("uihhh"),
    }

    Ok(())
}

fn make_basic(admiral: &mut AdmiralClient) -> AdmiralResult<()> {
    execute_destroy(admiral)?;

    let entity = FacEntChest::new(FacEntChestType::Active);
    admiral.execute_checked_command(entity.to_fac(0, &VPoint::zero()).to_lua().into_boxed())?;

    Ok(())
}

fn make_assembler_thru(admiral: &mut AdmiralClient) -> AdmiralResult<()> {
    execute_destroy(admiral)?;

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
    };
    for entity in farm.generate(VPoint::new(5, 5)) {
        admiral.execute_checked_command(entity.to_blueprint().to_lua().into_boxed())?;
    }

    Ok(())
}

fn make_belt_bettel(admiral: &mut AdmiralClient) -> AdmiralResult<()> {
    execute_destroy(admiral)?;

    let mut belt = FacBlkBettelBelt::new(
        FacEntBeltType::Basic,
        VPoint::new(5, 5),
        FacDirectionQuarter::South,
    );
    belt.add_straight(5);
    belt.add_turn90(false);
    belt.add_straight_underground(5);
    belt.add_turn90(true);
    belt.add_straight(5);

    for entity in belt.to_fac() {
        admiral.execute_checked_command(entity.to_blueprint().to_lua().into_boxed())?;
    }

    Ok(())
}

fn make_rail_spiral_90(admiral: &mut AdmiralClient) -> AdmiralResult<()> {
    execute_destroy(admiral)?;

    let mut existing_points: Vec<FacBpPosition> = Vec::new();
    let origin: VPoint = VPoint::zero();
    for clockwise in [
        true,  //
        false, //
    ] {
        let hope1 = RailHopeSingle::new(origin, FacDirectionQuarter::North);
        let hope2 = RailHopeSingle::new(origin, FacDirectionQuarter::South);
        let hope3 = RailHopeSingle::new(origin, FacDirectionQuarter::East);
        let hope4 = RailHopeSingle::new(origin, FacDirectionQuarter::West);
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

            for entity in hope.to_fac() {
                let bpfac = entity.to_blueprint();
                let bppos = &bpfac.position;
                if existing_points.contains(bppos) {
                    continue;
                } else {
                    existing_points.push(bppos.clone());
                }
                admiral.execute_checked_command(bpfac.to_lua().into_boxed())?;
            }
        }
    }

    Ok(())
}

fn make_rail_shift_45(admiral: &mut AdmiralClient) -> AdmiralResult<()> {
    execute_destroy(admiral)?;

    let mut existing_points = Vec::new();
    let origin = VPoint::zero();
    for clockwise in [false /*, false*/] {
        let hope1 = RailHopeSingle::new(origin, FacDirectionQuarter::North);
        let hope2 = RailHopeSingle::new(origin, FacDirectionQuarter::South);
        let hope3 = RailHopeSingle::new(origin, FacDirectionQuarter::East);
        let hope4 = RailHopeSingle::new(origin, FacDirectionQuarter::West);
        for mut hope in [hope1, hope2, hope3, hope4] {
            hope.add_straight(2);
            hope.add_shift45(clockwise, 3);
            hope.add_straight(2);

            for entity in hope.to_fac() {
                let bpfac = entity.to_blueprint();
                let bppos = &bpfac.position;
                if existing_points.contains(bppos) {
                    continue;
                } else {
                    existing_points.push(bppos.clone());
                }
                admiral.execute_checked_command(bpfac.to_lua().into_boxed())?;
            }
        }
    }

    Ok(())
}

fn make_rail_dual_turning(admiral: &mut AdmiralClient) -> AdmiralResult<()> {
    execute_destroy(admiral)?;

    let mut existing_points = Vec::new();

    for clockwise in [true, false] {
        for direction in [
            FacDirectionQuarter::North,
            FacDirectionQuarter::East,
            FacDirectionQuarter::South,
            FacDirectionQuarter::West,
        ] {
            let mut hope = RailHopeDual::new(VPoint::zero(), direction);
            hope.add_straight(5);
            hope.add_turn90(clockwise);
            hope.add_straight(5);
            hope.add_straight(5);

            for entity in hope.to_fac() {
                let bpfac = entity.to_blueprint();
                let bppos = &bpfac.position;
                if existing_points.contains(bppos) {
                    continue;
                } else {
                    existing_points.push(bppos.clone());
                }
                admiral.execute_checked_command(bpfac.to_lua().into_boxed())?;
            }
        }
    }

    Ok(())
}

fn make_rail_dual_powered(admiral: &mut AdmiralClient) -> AdmiralResult<()> {
    execute_destroy(admiral)?;

    for direction in [
        FacDirectionQuarter::North,
        // FacDirectionQuarter::East,
        // FacDirectionQuarter::South,
        // FacDirectionQuarter::West,
    ] {
        let origin = VPoint::zero().move_direction(&direction, 6);

        let mut hope = RailHopeDual::new(origin, direction);
        hope.add_straight_section();
        hope.add_turn90(true);
        hope.add_straight_section();
        hope.add_turn90(true);
        hope.add_straight_section();

        for entity in hope.to_fac() {
            let bpfac = entity.to_blueprint();
            // let bppos = &bpfac.position;
            // if existing_points.contains(bppos) {
            //     continue;
            // } else {
            //     existing_points.push(bppos.clone());
            // }
            admiral.execute_checked_command(bpfac.to_lua().into_boxed())?;
        }
    }

    Ok(())
}

fn make_rail_station(admiral: &mut AdmiralClient) -> AdmiralResult<()> {
    execute_destroy(admiral)?;

    let station = FacBlkRailStation {
        wagons: 3,
        front_engines: 2,
        chests: Some(FacEntChestType::Steel),
        inserter: FacEntInserterType::Basic,
        is_east: true,
        // is_east: false,
        is_up: true,
        // is_up: false,
        is_input: true,
    };
    for entity in station.generate(VPOINT_ZERO) {
        admiral.execute_checked_command(entity.to_blueprint().to_lua().into_boxed())?;
    }
    Ok(())
}

fn make_rail_loop(admiral: &mut AdmiralClient) -> AdmiralResult<()> {
    execute_destroy(admiral)?;

    let origin = VPoint::zero();

    admiral.execute_checked_command(
        BlueprintItem::new(
            FacEntInfinityPower::new().into_boxed(),
            origin.move_xy(4, 2),
        )
        .to_blueprint()
        .to_lua()
        .into_boxed(),
    )?;

    let mut rail_loop = FacBlkRailLoop::new(FacBlkRailLoopProps {
        wagons: 2,
        front_engines: 2,
        origin,
        origin_direction: FacDirectionQuarter::West,
        chest_type: Some(FacEntChestType::Infinity),
        inserter_type: FacEntInserterType::Stack,
        is_start_input: true,
    });
    rail_loop.add_turn90(false);
    rail_loop.add_straight();
    rail_loop.add_turn90(false);

    for entity in rail_loop.to_fac() {
        admiral.execute_checked_command(entity.to_blueprint().to_lua().into_boxed())?;
    }

    Ok(())
}

fn execute_destroy(admiral: &mut AdmiralClient) -> AdmiralResult<()> {
    let command = FacDestroy::new_filtered(
        150,
        FacEntityName::iter_exhaustive(None)
            .map(|v| v.to_fac_name())
            .collect(),
    );
    // Do not use, this deletes mine resource tiles
    // let command = FacDestroy::new_everything(50);
    admiral.execute_checked_command(command.into_boxed())?;

    Ok(())
}
