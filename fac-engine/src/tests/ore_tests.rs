use std::rc::Rc;

use crate::{
    blueprint::output::FacItemOutput,
    common::vpoint::VPOINT_TEN,
    game_blocks::{block::FacBlock, mine_ore::FacBlkMineOre},
    game_entities::{belt::FacEntBeltType, direction::FacDirectionQuarter},
};

pub fn make_mine(output: Rc<FacItemOutput>) {
    let block = FacBlkMineOre {
        height: 2,
        width: 5,
        build_direction: FacDirectionQuarter::East,
        belt: FacEntBeltType::Basic,
        output,
    };
    block.generate(VPOINT_TEN);
}
