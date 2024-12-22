use facto_loop_miner_common::log_init;
use facto_loop_miner_fac_engine::blueprint::bpfac::position::FacBpPosition;
use facto_loop_miner_fac_engine::common::names::FacEntityName;
use facto_loop_miner_fac_engine::common::vpoint::VPOINT_ZERO;
use facto_loop_miner_fac_engine::game_blocks::belt_bettel::FacBlkBettelBelt;
use facto_loop_miner_fac_engine::game_blocks::rail_hope::RailHopeAppender;
use facto_loop_miner_fac_engine::game_blocks::rail_hope_dual::RailHopeDual;
use facto_loop_miner_fac_engine::game_blocks::rail_hope_single::RailHopeSingle;
use facto_loop_miner_fac_engine::game_blocks::rail_station::FacBlkRailStation;
use facto_loop_miner_fac_engine::game_entities::direction::FacDirectionQuarter;
use facto_loop_miner_fac_engine::game_entities::electric_large::FacEntElectricLargeType;
use facto_loop_miner_fac_engine::game_entities::electric_mini::FacEntElectricMiniType;
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

    match 8 {
        1 => make_basic(&mut client)?,
        2 => make_assembler_thru(&mut client)?,
        3 => make_belt_bettel(&mut client)?,
        4 => make_rail_spiral_90(&mut client)?,
        5 => make_rail_shift_45(&mut client)?,
        6 => make_rail_dual_turning(&mut client)?,
        7 => make_rail_dual_powered(&mut client)?,
        8 => make_rail_station(&mut client)?,
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
            let bppos = &bpfac.position;
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

    let station = FacBlkRailStation::new(3, Some(FacEntChestType::Steel), 3);
    for entity in station.generate(VPOINT_ZERO) {
        admiral.execute_checked_command(entity.to_blueprint().to_lua().into_boxed())?;
    }
    Ok(())
}

fn execute_destroy(admiral: &mut AdmiralClient) -> AdmiralResult<()> {
    let command = FacDestroy::new_filtered(
        90,
        [
            FacEntityName::Lamp,
            FacEntityName::RailStraight,
            FacEntityName::RailCurved,
            FacEntityName::Assembler(FacTier::Tier1),
            FacEntityName::Assembler(FacTier::Tier2),
            FacEntityName::Assembler(FacTier::Tier3),
            FacEntityName::Inserter(FacEntInserterType::Burner),
            FacEntityName::Inserter(FacEntInserterType::Basic),
            FacEntityName::Inserter(FacEntInserterType::Long),
            FacEntityName::Inserter(FacEntInserterType::Fast),
            FacEntityName::Inserter(FacEntInserterType::Filter),
            FacEntityName::Inserter(FacEntInserterType::Stack),
            FacEntityName::Inserter(FacEntInserterType::StackFilter),
            FacEntityName::Chest(FacEntChestType::Wood),
            FacEntityName::Chest(FacEntChestType::Iron),
            FacEntityName::Chest(FacEntChestType::Steel),
            FacEntityName::Chest(FacEntChestType::Active),
            FacEntityName::Chest(FacEntChestType::Passive),
            FacEntityName::Chest(FacEntChestType::Storage),
            FacEntityName::Chest(FacEntChestType::Buffer),
            FacEntityName::Chest(FacEntChestType::Requestor),
            FacEntityName::ElectricMini(FacEntElectricMiniType::Small),
            FacEntityName::ElectricMini(FacEntElectricMiniType::Medium),
            FacEntityName::ElectricLarge(FacEntElectricLargeType::Big),
            FacEntityName::ElectricLarge(FacEntElectricLargeType::Substation),
            FacEntityName::TrainStop,
            FacEntityName::Beacon,
            FacEntityName::Radar,
            FacEntityName::Roboport,
            FacEntityName::BeltTransport(FacEntBeltType::Basic),
            FacEntityName::BeltTransport(FacEntBeltType::Fast),
            FacEntityName::BeltTransport(FacEntBeltType::Express),
            FacEntityName::BeltUnder(FacEntBeltType::Basic),
            FacEntityName::BeltUnder(FacEntBeltType::Fast),
            FacEntityName::BeltUnder(FacEntBeltType::Express),
            FacEntityName::BeltSplit(FacEntBeltType::Basic),
            FacEntityName::BeltSplit(FacEntBeltType::Fast),
            FacEntityName::BeltSplit(FacEntBeltType::Express),
            FacEntityName::InfinityPower,
            FacEntityName::Locomotive,
            FacEntityName::CargoWagon,
        ]
        .into_iter()
        .map(|v| v.to_fac_name())
        .collect(),
    );
    // Do not use, this deletes mine resource tiles
    // let command = FacDestroy::new_everything(50);
    admiral.execute_checked_command(command.into_boxed())?;

    Ok(())
}
