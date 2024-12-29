use std::rc::Rc;

use crate::{
    admiral::err::AdmiralResult,
    blueprint::output::FacItemOutput,
    common::vpoint::VPoint,
    game_blocks::{assembler_thru::FacBlkAssemblerThru, block::FacBlock},
    game_entities::{
        assembler::FacEntAssembler, belt::FacEntBeltType, inserter::FacEntInserterType,
        module::FacModule, tier::FacTier,
    },
};

pub fn make_assembler_thru(output: Rc<FacItemOutput>) -> AdmiralResult<()> {
    let farm = FacBlkAssemblerThru {
        assembler: FacEntAssembler::new(FacTier::Tier3, "copper-cable".into(), [
            Some(FacModule::Speed(FacTier::Tier3)),
            Some(FacModule::Speed(FacTier::Tier3)),
            Some(FacModule::Speed(FacTier::Tier3)),
        ]),
        belt_type: FacEntBeltType::Fast,
        inserter_input: FacEntInserterType::Fast,
        inserter_output: FacEntInserterType::Basic,
        width: 4,
        height: 3,
        output: output.clone(),
    };
    farm.generate(VPoint::new(5, 5));

    Ok(())
}
