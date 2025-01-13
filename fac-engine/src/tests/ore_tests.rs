use std::rc::Rc;

use crate::{
    admiral::generators::xy_grid_vpoint,
    blueprint::output::FacItemOutput,
    common::vpoint::{VPOINT_TEN, VPoint},
    game_blocks::{
        block::FacBlock2,
        mine_ore::FacBlkMineOre,
        rail_station::{FacBlkRailStation, FacExtDelivery},
    },
    game_entities::{
        belt::FacEntBeltType, direction::FacDirectionQuarter,
        electric_large::FacEntElectricLargeType, infinity_power::FacEntInfinityPower,
        inserter::FacEntInserterType, resource::FacEntResourceType,
    },
};

pub fn make_mine(output: Rc<FacItemOutput>) {
    output.writei(FacEntInfinityPower::new(), VPoint::new(8, 8));
    output.writei(FacEntElectricLargeType::Big.entity(), VPoint::new(7, 7));

    for pos in xy_grid_vpoint(VPOINT_TEN, 70, 20, 1) {
        output.writei(FacEntResourceType::IronOre.entity(), pos.point());
    }

    let block = FacBlkMineOre {
        height: 3,
        width: 10,
        build_direction: FacDirectionQuarter::East,
        belt: FacEntBeltType::Basic,
        drill_modules: Default::default(),
        output,
    };
    let belts = block.generate(VPOINT_TEN);
    for mut belt in belts {
        belt.add_straight(75);
    }
}

pub fn make_mine_and_rail(output: Rc<FacItemOutput>) {
    // let block = FacBlkMineOre {
    //     height: 3,
    //     width: 5,
    //     build_direction: FacDirectionQuarter::North,
    //     belt: FacEntBeltType::Basic,
    //     drill_modules: Default::default(),
    //     output: output.clone(),
    // };
    // block.generate(VPOINT_TEN);

    let test = FacBlkRailStation {
        name: "test".into(),
        wagons: 2,
        front_engines: 2,
        delivery: FacExtDelivery::Belt(FacEntBeltType::Basic),
        fuel_inserter: None,
        fuel_inserter_chest: None,
        inserter: FacEntInserterType::Basic,
        schedule: None,
        is_create_train: true,
        is_east: true,
        is_up: true,
        is_input: true,
        output: output.clone(),
    };
    test.generate(VPoint::new(10, 50));
}
