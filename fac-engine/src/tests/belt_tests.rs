use std::rc::Rc;

use crate::blueprint::output::ContextLevel;
use crate::common::vpoint::VPoint;
use crate::common::vpoint_direction::VPointDirectionQ;
use crate::game_blocks::belt_array::FacBlkBettelArray;
use crate::game_blocks::belt_train_unload::UnloadMode;
use crate::game_blocks::block::FacBlockFancy;
use crate::{
    admiral::err::AdmiralResult,
    blueprint::output::FacItemOutput,
    common::vpoint::{VPOINT_TEN, VPOINT_ZERO},
    game_blocks::{
        belt_bettel::FacBlkBettelBelt, belt_combiner::FacBlkBeltCombiner,
        belt_train_unload::FacBlkBeltTrainUnload, block::FacBlock,
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
        wagons: 3,
        padding_unmerged: 0, // 2,
        padding_above: 1,
        padding_after: 1,
        turn_clockwise: true,
        origin: VPointDirectionQ(VPOINT_ZERO, FacDirectionQuarter::East),
        mode: UnloadMode::Turn,
    }
    .generate();

    Ok(())
}

pub fn make_belt_combiner(output: Rc<FacItemOutput>) {
    let block =
        FacBlkBeltCombiner::new_wavy(FacEntBeltType::Basic, FacDirectionQuarter::East, 10, output);
    block.generate(VPOINT_TEN);
}

pub fn make_belt_grid(output: Rc<FacItemOutput>) {
    // let block = FacBlkBeltCloud {
    //     belt_input: FacEntBeltType::Basic,
    //     belt_output: FacEntBeltType::Fast,
    //     sources:
    //     output: output.clone(),
    //     origin_direction: FacDirectionQuarter::East,
    // };
    todo!()
    // block.generate(VPOINT_TEN);
}

pub fn make_belt_array(output: Rc<FacItemOutput>) {
    let mut origin = VPoint::new(15, 15);
    for direction in [
        FacDirectionQuarter::North,
        FacDirectionQuarter::South,
        FacDirectionQuarter::East,
        FacDirectionQuarter::West,
    ] {
        for rotation in [true, false] {
            let context = output.context_handle(
                ContextLevel::Block,
                format!("{direction}{}", if rotation { "clk" } else { "ccw" }),
            );

            let mut belts = Vec::new();
            for i in 0..5 {
                let context = output.context_handle(ContextLevel::Micro, "init".into());
                belts.push(FacBlkBettelBelt::new(
                    FacEntBeltType::Basic,
                    match direction {
                        FacDirectionQuarter::North | FacDirectionQuarter::South => origin.move_x(i),
                        FacDirectionQuarter::West => origin.move_x(0).move_y(i),
                        FacDirectionQuarter::East => origin.move_y(i),
                    },
                    direction,
                    output.clone(),
                ))
            }

            let mut array = FacBlkBettelArray::new(belts);
            let context = output.context_handle(ContextLevel::Micro, "arr".into());
            array.add_straight(2);
            drop(context);

            let context = output.context_handle(ContextLevel::Micro, "tur".into());
            if rotation {
                array.add_turn90_clk();
            } else {
                array.add_turn90_ccw();
            }
            drop(context);

            origin += VPoint::new(15, 0);
        }
    }
}
