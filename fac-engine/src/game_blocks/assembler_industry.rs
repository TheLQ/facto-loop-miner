use std::rc::Rc;

use crate::{
    blueprint::output::{ContextLevel, FacItemOutput},
    common::{names::FacEntityName, vpoint::VPoint},
    game_entities::{
        assembler::{FacEntAssembler, FacEntAssemblerModSlice},
        belt::FacEntBeltType,
        direction::FacDirectionQuarter,
        inserter::FacEntInserterType,
        tier::FacTier,
    },
};

use super::{assembler_thru::FacBlkAssemblerThru, belt_bettel::FacBlkBettelBelt, block::FacBlock};

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

        let mut output_belts = self.place_input_belts(origin);

        for thru in &self.thru {
            self.place_thru(thru, output_belts.remove(0));
        }
    }
}

impl FacBlkIndustry {
    fn place_input_belts(&self, origin: VPoint) -> Vec<VPoint> {
        let mut output_belts = Vec::new();

        let total_input_belts: usize = self.thru.iter().map(|v| v.input_belts).sum();

        let mut cur_belt_count: usize = 0;
        let mut height_offset = 0;
        for (thru_num, thru) in self.thru.iter().enumerate() {
            for thru_belt_num in 0..thru.input_belts {
                let bettel_origin = origin.move_y_usize(cur_belt_count);

                let mut belt = FacBlkBettelBelt::new(
                    self.belt,
                    bettel_origin,
                    FacDirectionQuarter::East,
                    self.output.clone(),
                );

                // arbitrary buffer
                belt.add_straight(2);

                // start turn buffering
                belt.add_straight(total_input_belts - cur_belt_count);

                // going down
                belt.add_turn90(true);
                belt.add_straight(height_offset);

                // end turn buffering
                belt.add_turn90(false);
                belt.add_straight(cur_belt_count);

                cur_belt_count += 1;

                if thru_belt_num == 0 {
                    output_belts.push(belt.next_insert_position());
                }
            }

            // go past the whole block
            height_offset += FacBlkAssemblerThru::total_height(thru.height) - thru.input_belts;

            // arbitrary buffer
            height_offset += 2;
        }
        output_belts
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
