use std::rc::Rc;

use crate::{
    admiral::err::AdmiralResult,
    blueprint::output::FacItemOutput,
    common::vpoint::{VPOINT_TEN, VPOINT_ZERO},
    game_blocks::{
        belt_bettel::FacBlkBettelBelt,
        belt_combiner::{FacBlkBeltCombiner, FacExtCombinerStage},
        belt_train_unload::FacBlkBeltTrainUnload,
        block::FacBlock,
    },
    game_entities::{belt::FacEntBeltType, direction::FacDirectionQuarter},
};

pub fn make_belt_bettel(output: Rc<FacItemOutput>) -> AdmiralResult<()> {
    let direction = FacDirectionQuarter::North;

    let mut belt =
        FacBlkBettelBelt::new(FacEntBeltType::Basic, VPOINT_TEN, direction, output.clone());
    // belt.add_straight(5);
    // belt.add_turn90(false);
    // belt.add_straight_underground(5);
    // belt.add_turn90(true);
    belt.add_straight(1);
    belt.add_split(true);
    let mut other_belt = belt.belt_for_splitter();
    other_belt.add_straight(5);
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
    belt.add_straight(1);
    belt.add_split(false);
    let mut other_belt = belt.belt_for_splitter();
    other_belt.add_straight(5);
    belt.add_straight(1);

    Ok(())
}

pub fn make_belt_bettel_train_unload(output: Rc<FacItemOutput>) -> AdmiralResult<()> {
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

pub fn make_belt_combiner(output: Rc<FacItemOutput>) {
    let block = FacBlkBeltCombiner {
        input_belts: 5,
        belt: FacEntBeltType::Basic,
        layout: FacExtCombinerStage::Fixed(4),
        direction: FacDirectionQuarter::North,
        output,
    };
    block.generate(VPOINT_TEN);
}
