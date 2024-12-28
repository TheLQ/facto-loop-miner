use std::rc::Rc;

use crate::{
    blueprint::output::FacItemOutput,
    common::vpoint::VPoint,
    game_entities::{belt::FacEntBeltType, direction::FacDirectionQuarter},
};

use super::{
    belt_bettel::FacBlkBettelBelt,
    belt_combiner::{FacBlkBeltCombiner, FacExtCombinerStage},
    block::FacBlock,
};

pub struct FacBlkBeltGrid {
    pub belt_type: FacEntBeltType,
    pub belts_input: usize,
    pub belts_output: usize,
    pub origin_direction: FacDirectionQuarter,
    pub output: Rc<FacItemOutput>,
}

impl FacBlock for FacBlkBeltGrid {
    fn generate(&self, origin: VPoint) {
        let belts = self.place_loading_belts(origin);
        for belt in belts {
            self.place_combiner_row(&belt);
        }
    }
}

impl FacBlkBeltGrid {
    fn place_loading_belts(&self, origin: VPoint) -> Vec<FacBlkBettelBelt> {
        let mut belts = Vec::new();
        for input_num in 0..self.belts_input {
            let mut other_belt = FacBlkBettelBelt::new(
                self.belt_type,
                origin.move_direction_sideways_usz(self.origin_direction, input_num),
                self.origin_direction,
                self.output.clone(),
            );

            let clock = false;
            other_belt.add_straight(self.belts_input - input_num);
            other_belt.add_turn90(!clock);
            other_belt.add_straight((input_num) * ((self.belts_output * 2) + 1));
            other_belt.add_turn90(clock);
            other_belt.add_straight(input_num);
            belts.push(other_belt);
        }
        belts
    }

    fn place_combiner_row(&self, source_belt: &FacBlkBettelBelt) {
        let combiner = FacBlkBeltCombiner {
            belt: self.belt_type,
            direction: self.origin_direction,
            layout: FacExtCombinerStage::FixedOutputBelts(self.belts_output),
            output: self.output.clone(),
        };
        combiner.generate(source_belt.next_insert_position());
    }
}
