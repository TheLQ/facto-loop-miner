use std::rc::Rc;

use crate::{
    blueprint::output::{ContextLevel, FacItemOutput},
    common::vpoint::VPoint,
    game_entities::{
        belt::FacEntBeltType,
        belt_split::{FacEntBeltSplitPriority, FacExtPriority},
        direction::FacDirectionQuarter,
    },
};

use super::{belt_bettel::FacBlkBettelBelt, block::FacBlock};

/// Creates splitter array and passthrough to
pub struct FacBlkBeltCombiner {
    pub belt: FacEntBeltType,
    pub direction: FacDirectionQuarter,
    pub output_belt_targets: Vec<usize>,
    pub output: Rc<FacItemOutput>,
}

impl FacBlock for FacBlkBeltCombiner {
    fn generate(&self, origin: VPoint) {
        let _ = &mut self
            .output
            .context_handle(ContextLevel::Block, "combiner".into());
        self.generate_fixed(origin, true)
    }
}

impl FacBlkBeltCombiner {
    pub fn new_wavy(
        belt: FacEntBeltType,
        direction: FacDirectionQuarter,
        total: usize,
        output: Rc<FacItemOutput>,
    ) -> Self {
        let mut output_belt_targets = Vec::new();
        for i in 0..total {
            output_belt_targets.push(i);
        }

        Self {
            belt,
            direction,
            output_belt_targets,
            output,
        }
    }

    fn generate_fixed(&self, origin: VPoint, clockwise: bool) {
        let mut belts = self.place_splits(origin, clockwise);
        self.place_fill(&mut belts);
        self.place_output_skips(&mut belts, clockwise);
    }

    fn place_splits(&self, origin: VPoint, clockwise: bool) -> Vec<FacBlkBettelBelt> {
        let output_priority = FacExtPriority::Left;
        let source_belt =
            FacBlkBettelBelt::new(self.belt, origin, self.direction, self.output.clone());

        // build tree of splits
        let mut belts_stack = vec![source_belt];
        for output_belt in 0..self.output_belt_targets.len() {
            let _ = &mut self
                .output
                .context_handle(ContextLevel::Micro, "splits".into());
            for side_belt in 0..2 {
                let cur_belt = belts_stack.last_mut().unwrap();
                if output_belt + 1 == self.output_belt_targets.len() && side_belt == 1 {
                    break;
                }
                cur_belt.add_split_priority(
                    clockwise,
                    FacEntBeltSplitPriority {
                        input: FacExtPriority::None,
                        output: output_priority,
                    },
                );

                let other_belt = cur_belt.belt_for_splitter();
                belts_stack.push(other_belt);
            }
        }

        belts_stack
    }

    fn place_fill(&self, belts_stack: &mut [FacBlkBettelBelt]) {
        // fill depth output belts
        let belt_adjustment = belts_stack.len().saturating_sub(2);
        for (i, belt) in belts_stack.iter_mut().enumerate() {
            let _ = &mut self
                .output
                .context_handle(ContextLevel::Micro, "fill".into());
            belt.add_straight(belt_adjustment.saturating_sub(i));
        }

        // padding
        // for belt in belts_stack.iter_mut() {
        //     belt.add_straight(3);
        // }
    }

    fn place_output_skips(&self, belts_stack: &mut [FacBlkBettelBelt], clockwise: bool) {
        // add skips
        for (output_belt_num, [first, last]) in belts_stack.iter_mut().array_chunks().enumerate() {
            let _ = &mut self
                .output
                .context_handle(ContextLevel::Micro, format!("ends{output_belt_num}"));

            let target_output = self.output_belt_targets[output_belt_num];

            let mut cur_index = 0;
            while cur_index < target_output.saturating_sub(1) {
                first.add_straight_underground(4);
                last.add_straight_underground(4);
                cur_index += 2;
            }

            while cur_index < target_output {
                first.add_straight_underground(1);
                last.add_straight_underground(1);
                cur_index += 1;
            }

            first.add_straight(1);

            last.add_straight_underground(1);
            last.add_turn90(!clockwise);
            last.add_turn90(!clockwise);
            last.add_straight(1);
        }
    }
}
