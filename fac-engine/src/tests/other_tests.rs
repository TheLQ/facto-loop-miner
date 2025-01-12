use std::rc::Rc;

use crate::{
    blueprint::output::FacItemOutput,
    common::vpoint::VPOINT_TEN,
    game_blocks::{block::FacBlock, solar_bath::FacBlkSolarBath},
};

pub fn make_solar_bath_test(output: Rc<FacItemOutput>) {
    let block = FacBlkSolarBath {
        width: 3,
        height: 3,
        output,
    };
    block.generate(VPOINT_TEN);
}
