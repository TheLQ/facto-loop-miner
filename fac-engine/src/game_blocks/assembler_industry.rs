use std::rc::Rc;

use crate::{
    blueprint::output::{ContextLevel, FacItemOutput},
    common::{names::FacEntityName, vpoint::VPoint},
    game_entities::{
        assembler::{FacEntAssembler, FacEntAssemblerModSlice},
        belt::FacEntBeltType,
        inserter::FacEntInserterType,
        tier::FacTier,
    },
};

use super::{assembler_thru::FacBlkAssemblerThru, block::FacBlock};

pub struct FacBlkIndustry {
    pub belt: FacEntBeltType,
    pub inserter_input: FacEntInserterType,
    pub inserter_output: FacEntInserterType,
    pub assembler_tier: FacTier,
    pub assembler_modules: FacEntAssemblerModSlice,
    pub thru: Vec<IndustryThru>,
    pub output: Rc<FacItemOutput>,
}

#[derive(Debug)]
pub struct IndustryThru {
    pub recipe: FacEntityName,
    pub input_belts: usize,
    pub width: usize,
    pub height: usize,
    pub custom_inserter_input: Option<FacEntInserterType>,
    pub custom_inserter_output: Option<FacEntInserterType>,
    pub custom_assembler_modules: Option<FacEntAssemblerModSlice>,
}

impl FacBlock for FacBlkIndustry {
    fn generate(&self, origin: VPoint) {
        let _ = &mut self
            .output
            .context_handle(ContextLevel::Block, "Industry".into());
        let mut thru_origin = origin;
        for thru in &self.thru {
            thru_origin =
                thru_origin.move_y_usize(FacBlkAssemblerThru::total_height(thru.height) + 1);

            self.place_thru(thru, thru_origin);
        }
    }
}

impl FacBlkIndustry {
    fn place_input_belts(&self, thru: &IndustryThru, origin: VPoint) {
        for belt in 0..thru.input_belts {}
    }

    fn place_thru(&self, thru: &IndustryThru, origin: VPoint) {
        let block = FacBlkAssemblerThru {
            assembler: FacEntAssembler::new(
                self.assembler_tier,
                thru.recipe.clone(),
                thru.custom_assembler_modules
                    .unwrap_or(self.assembler_modules),
            ),
            belt_type: self.belt,
            height: thru.height,
            width: thru.width,
            inserter_input: thru.custom_inserter_input.unwrap_or(self.inserter_input),
            inserter_output: thru.custom_inserter_output.unwrap_or(self.inserter_output),
            output: self.output.clone(),
        };
        block.generate(origin);
    }

    fn place_output_belts() {}
}
