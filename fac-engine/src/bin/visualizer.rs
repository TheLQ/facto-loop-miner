use std::rc::Rc;

use facto_loop_miner_common::log_init_debug;
use facto_loop_miner_fac_engine::blueprint::output::FacItemOutput;
use facto_loop_miner_fac_engine::common::names::FacEntityName;
use facto_loop_miner_fac_engine::game_blocks::block::FacBlock2;
use facto_loop_miner_fac_engine::game_blocks::rail_hope::RailHopeAppender;
use facto_loop_miner_fac_engine::game_blocks::rail_hope_dual::RailHopeDual;
use facto_loop_miner_fac_engine::game_blocks::rail_hope_single::RailHopeSingle;
use facto_loop_miner_fac_engine::game_blocks::rail_station::FacExtDelivery;
use facto_loop_miner_fac_engine::game_entities::direction::FacDirectionQuarter;
use facto_loop_miner_fac_engine::{
    common::vpoint::VPoint,
    game_blocks::{
        assembler_cell::{FacBlkAssemblerCell, FacBlkAssemblerCellEntry},
        beacon_farm::FacBlkBeaconFarm,
        block::FacBlock,
        rail_station::FacBlkRailStation,
        robo_farm::FacBlkRobofarm,
        terapower::FacBlkTerapower,
    },
    game_entities::{
        assembler::FacEntAssembler,
        chest::{FacEntChest, FacEntChestType},
        inserter::FacEntInserterType,
        module::FacModule,
        tier::FacTier,
    },
    visualizer::visualizer::visualize_blueprint,
};

fn main() {
    log_init_debug();

    let output = FacItemOutput::new_blueprint().into_rc();

    match 8 {
        2 => basic_build_gen(output.clone()),
        3 => basic_build_terapower(output.clone()),
        4 => basic_build_beacon_farm(output.clone()),
        5 => basic_build_robo_farm(output.clone()),
        7 => basic_build_rail_hope_single(output.clone()),
        8 => basic_build_rail_hope_dual(output.clone()),
        _ => panic!("asdf"),
    }

    visualize_blueprint(&output.consume_rc().into_blueprint_contents());

    // let res: Vec<FacSurfaceCreateEntity> = bp_contents
    //     .entities()
    //     .iter()
    //     .enumerate()
    //     .map(|(i, v)| v.entity().to_fac_usize(i, v.position()).to_lua())
    //     .collect();

    // let blueprint = Blueprint::new(bp_contents);

    // let res = encode_blueprint_to_string(&blueprint.to_fac()).unwrap();
    // println!("bp {}", res);
}

fn basic_build_gen(output: Rc<FacItemOutput>) {
    let station = FacBlkRailStation {
        name: "test".into(),
        wagons: 3,
        front_engines: 2,
        delivery: FacExtDelivery::Chest(FacEntChestType::Passive),
        inserter: FacEntInserterType::Basic,
        fuel_inserter: None,
        fuel_inserter_chest: None,
        schedule: None,
        is_east: true,
        is_up: true,
        is_input: true,
        is_create_train: true,
        is_electric_initial: true,
        output,
    };
    station.generate(VPoint::new(5, 5));
}

fn basic_build_terapower(output: Rc<FacItemOutput>) {
    let station = FacBlkTerapower::new(3, 2, output);
    station.generate(VPoint::new(5, 5));
}

fn basic_build_beacon_farm(output: Rc<FacItemOutput>) {
    let station = FacBlkBeaconFarm {
        inner_cell_size: 2,
        width: 3,
        height: 3,
        module: FacModule::Speed(FacTier::Tier3),
        cell: Some(FacBlkAssemblerCell {
            assembler: FacEntAssembler::new(
                FacTier::Tier1,
                FacEntityName::CopperCable,
                Default::default(),
            ),
            side_bottom: [
                Some(FacBlkAssemblerCellEntry {
                    chest: FacEntChest::new(FacEntChestType::Requestor),
                    inserter: FacEntInserterType::Fast,
                    is_loader: false,
                }),
                Some(FacBlkAssemblerCellEntry {
                    chest: FacEntChest::new(FacEntChestType::Requestor),
                    inserter: FacEntInserterType::Fast,
                    is_loader: false,
                }),
                Some(FacBlkAssemblerCellEntry {
                    chest: FacEntChest::new(FacEntChestType::Passive),
                    inserter: FacEntInserterType::Fast,
                    is_loader: true,
                }),
            ],
            side_right: [
                Some(FacBlkAssemblerCellEntry {
                    chest: FacEntChest::new(FacEntChestType::Passive),
                    inserter: FacEntInserterType::Fast,
                    is_loader: true,
                }),
                None,
                None,
            ],
            is_big_power: true,
            output: output.clone(),
        }),
        output,
    };
    station.generate(VPoint::new(5, 5))
}

fn basic_build_robo_farm(output: Rc<FacItemOutput>) {
    let farm = FacBlkRobofarm {
        width: 3,
        height: 3,
        is_row_depth_full: true,
        output,
    };
    farm.generate(VPoint::new(5, 5))
}

fn basic_build_rail_hope_single(output: Rc<FacItemOutput>) {
    let mut farm = RailHopeSingle::new(VPoint::new(6, 6), FacDirectionQuarter::East, output);
    farm.add_straight(5);
}

fn basic_build_rail_hope_dual(output: Rc<FacItemOutput>) {
    let mut farm = RailHopeDual::new(VPoint::new(6, 6), FacDirectionQuarter::East, output);
    farm.add_straight_section();
}
