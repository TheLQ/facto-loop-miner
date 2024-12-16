use facto_loop_miner_common::log_init;
use facto_loop_miner_fac_engine::{
    blueprint::{bpitem::BlueprintItem, contents::BlueprintContents},
    common::{entity::FacEntity, vpoint::VPoint},
    game_blocks::{
        assembler_cell::{FacBlkAssemblerCell, FacBlkAssemblerCellEntry},
        assembler_thru::FacBlkAssemblerThru,
        beacon_farm::FacBlkBeaconFarm,
        block::FacBlock,
        rail_station::FacBlkRailStation,
        robo_farm::FacBlkRobofarm,
        terapower::FacBlkTerapower,
    },
    game_entities::{
        assembler::FacEntAssembler,
        belt::FacEntBeltType,
        chest::{FacEntChest, FacEntityChestType},
        inserter::FacEntInserterType,
        module::FacModule,
        tier::FacTier,
    },
    visualizer::visualizer::visualize_blueprint,
};

fn main() {
    log_init(None);

    let mut bp_contents = BlueprintContents::new();

    match 6 {
        1 => basic_build_bp(&mut bp_contents),
        2 => basic_build_gen(&mut bp_contents),
        3 => basic_build_terapower(&mut bp_contents),
        4 => basic_build_beacon_farm(&mut bp_contents),
        5 => basic_build_robo_farm(&mut bp_contents),
        6 => basic_build_assembler_thru(&mut bp_contents),
        _ => panic!("asdf"),
    }

    visualize_blueprint(&bp_contents);

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

fn basic_build_bp(bp: &mut BlueprintContents) {
    {
        let entity = FacEntAssembler::new_basic(FacTier::Tier1, "something".into());
        bp.add_entity_each(BlueprintItem::new(entity.into_boxed(), VPoint::new(1, 1)));
    }

    {
        let entity = FacEntAssembler::new_basic(FacTier::Tier1, "something2".into());
        bp.add_entity_each(BlueprintItem::new(entity.into_boxed(), VPoint::new(1, 4)));
    }
}

fn basic_build_gen(bp: &mut BlueprintContents) {
    let station = FacBlkRailStation::new(3, Some(FacEntityChestType::Passive), 2);
    for entity in station.generate(VPoint::new(5, 5)) {
        bp.add_entity_each(entity);
    }
}

fn basic_build_terapower(bp: &mut BlueprintContents) {
    let station = FacBlkTerapower::new(3, 2);
    for entity in station.generate(VPoint::new(5, 5)) {
        bp.add_entity_each(entity);
    }
}

fn basic_build_beacon_farm(bp: &mut BlueprintContents) {
    let station = FacBlkBeaconFarm {
        inner_cell_size: 2,
        width: 3,
        height: 3,
        module: FacModule::Speed(FacTier::Tier3),
        cell: Some(FacBlkAssemblerCell {
            assembler: FacEntAssembler::new(
                FacTier::Tier1,
                "small-lamp".into(),
                Default::default(),
            ),
            side_bottom: [
                Some(FacBlkAssemblerCellEntry {
                    chest: FacEntChest::new(FacEntityChestType::Requestor),
                    inserter: FacEntInserterType::Fast,
                    is_loader: false,
                }),
                Some(FacBlkAssemblerCellEntry {
                    chest: FacEntChest::new(FacEntityChestType::Requestor),
                    inserter: FacEntInserterType::Fast,
                    is_loader: false,
                }),
                Some(FacBlkAssemblerCellEntry {
                    chest: FacEntChest::new(FacEntityChestType::Passive),
                    inserter: FacEntInserterType::Fast,
                    is_loader: true,
                }),
            ],
            side_right: [
                Some(FacBlkAssemblerCellEntry {
                    chest: FacEntChest::new(FacEntityChestType::Passive),
                    inserter: FacEntInserterType::Fast,
                    is_loader: true,
                }),
                None,
                None,
            ],
            is_big_power: true,
        }),
    };
    for entity in station.generate(VPoint::new(5, 5)) {
        bp.add_entity_each(entity);
    }
}

fn basic_build_robo_farm(bp: &mut BlueprintContents) {
    let farm = FacBlkRobofarm {
        width: 3,
        height: 3,
        is_row_depth_full: true,
    };
    for entity in farm.generate(VPoint::new(5, 5)) {
        bp.add_entity_each(entity);
    }
}

fn basic_build_assembler_thru(bp: &mut BlueprintContents) {
    let farm = FacBlkAssemblerThru {
        assembler: FacEntAssembler::new(FacTier::Tier1, "copper-wire".into(), Default::default()),
        belt_type: FacEntBeltType::Fast,
        inserter_type: FacEntInserterType::Fast,
        width: 2,
        height: 3,
    };
    for entity in farm.generate(VPoint::new(5, 5)) {
        bp.add_entity_each(entity);
    }
}
