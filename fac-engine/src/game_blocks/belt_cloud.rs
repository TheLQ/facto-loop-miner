use std::rc::Rc;

use crate::{
    blueprint::output::FacItemOutput,
    common::vpoint::VPoint,
    game_entities::{belt::FacEntBeltType, direction::FacDirectionQuarter},
};

use super::{belt_bettel::FacBlkBettelBelt, belt_combiner::FacBlkBeltCombiner, block::FacBlock};

pub struct FacBlkBeltCloud {
    pub belt_input: FacEntBeltType,
    pub belt_output: FacEntBeltType,
    pub belts_input: usize,
    pub belts_output: usize,
    pub origin_direction: FacDirectionQuarter,
    pub output: Rc<FacItemOutput>,
}

impl FacBlock for FacBlkBeltCloud {
    fn generate(&self, origin: VPoint) {
        let belts = self.place_loading_belts(origin);
        for belt in &belts {
            self.place_combiner_row(belt);
        }
        self.place_output_thru(&belts);
    }
}

impl FacBlkBeltCloud {
    fn place_loading_belts(&self, origin: VPoint) -> Vec<FacBlkBettelBelt> {
        let mut belts = Vec::new();
        for input_num in 0..self.belts_input {
            let mut other_belt = FacBlkBettelBelt::new(
                self.belt_input,
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
            belt: self.belt_input,
            direction: self.origin_direction,
            output_belt_order: todo!(),
            output: self.output.clone(),
        };
        combiner.generate(source_belt.next_insert_position());
    }

    fn place_output_thru(&self, belts_stack: &[FacBlkBettelBelt]) {
        let last_belt = belts_stack.last().unwrap();
        let pos = last_belt
            .next_insert_position()
            .move_direction_usz(self.origin_direction, (self.belts_output * 2) + 1)
            .move_direction_usz(self.origin_direction.rotate_once(), self.belts_output * 2);
        for belt_num in 0..self.belts_output {
            let mut belt = FacBlkBettelBelt::new(
                self.belt_output,
                pos.move_direction_usz(self.origin_direction, belt_num * 3),
                self.origin_direction.rotate_opposite(),
                self.output.clone(),
            );
            belt.add_straight(
                // Reaching each side takes 2 belts. Then passthru belt, then 1x empty spacer
                ((self.belts_output * 2) +2 )
                // span
                * self.belts_input,
            );
        }
    }
}
