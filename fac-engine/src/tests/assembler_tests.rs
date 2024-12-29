use std::rc::Rc;

use crate::{
    blueprint::output::FacItemOutput,
    common::{names::FacEntityName, vpoint::VPoint},
    game_blocks::{
        assembler_industry::{FacBlkIndustry, IndustryThru},
        assembler_thru::FacBlkAssemblerThru,
        block::FacBlock,
    },
    game_entities::{
        assembler::FacEntAssembler, belt::FacEntBeltType, inserter::FacEntInserterType,
        module::FacModule, tier::FacTier,
    },
};

pub fn make_assembler_thru(output: Rc<FacItemOutput>) {
    let farm = FacBlkAssemblerThru {
        assembler: FacEntAssembler::new(FacTier::Tier3, FacEntityName::CopperCable, [
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
}

pub fn make_industry(output: Rc<FacItemOutput>) {
    let farm = FacBlkIndustry {
        assembler_modules: Default::default(),
        assembler_tier: FacTier::Tier3,
        belt: FacEntBeltType::Basic,
        inserter_input: FacEntInserterType::Fast,
        inserter_output: FacEntInserterType::Basic,
        thru: Vec::from([
            IndustryThru {
                height: 1,
                input_belts: 2,
                recipe: FacEntityName::CopperCable,
                width: 2,
                custom_assembler_modules: None,
                custom_inserter_input: None,
                custom_inserter_output: None,
            },
            IndustryThru {
                height: 1,
                input_belts: 2,
                recipe: FacEntityName::IronGear,
                width: 5,
                custom_assembler_modules: None,
                custom_inserter_input: None,
                custom_inserter_output: None,
            },
        ]),
        output,
    };
    farm.generate(VPoint::new(5, 5));
}
