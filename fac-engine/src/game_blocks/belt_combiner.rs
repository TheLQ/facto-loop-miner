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

pub struct FacBlkBeltCombiner {
    pub input_belts: usize,
    pub belt: FacEntBeltType,
    pub layout: FacExtCombinerStage,
    pub direction: FacDirectionQuarter,
    pub output: Rc<FacItemOutput>,
}

pub enum FacExtCombinerStage {
    FixedOutputBelts(usize),
}

impl FacBlock for FacBlkBeltCombiner {
    fn generate(&self, origin: VPoint) {
        let _ = &mut self
            .output
            .context_handle(ContextLevel::Block, "combiner".into());
        match &self.layout {
            FacExtCombinerStage::FixedOutputBelts(output_belts) => {
                self.generate_fixed(origin, *output_belts, true)
            }
        };
    }
}

impl FacBlkBeltCombiner {
    fn generate_fixed(&self, origin: VPoint, output_belts: usize, clockwise: bool) {
        let output_priority = FacExtPriority::Left;

        let source_belt =
            FacBlkBettelBelt::new(self.belt, origin, self.direction, self.output.clone());

        // build tree of splits
        let mut belts_stack = vec![source_belt];
        for output_belt in 0..output_belts {
            let _ = &mut self
                .output
                .context_handle(ContextLevel::Micro, "splits".into());
            for side_belt in 0..2 {
                let cur_belt = belts_stack.last_mut().unwrap();
                cur_belt.add_split_priority(clockwise, FacEntBeltSplitPriority {
                    input: FacExtPriority::None,
                    output: if output_belt == 0 && side_belt == 0 {
                        output_priority.flip()
                    } else if output_belt == (output_belts - 1) && side_belt == 1 {
                        FacExtPriority::None
                    } else {
                        output_priority
                    },
                });

                let other_belt = cur_belt.belt_for_splitter();
                belts_stack.push(other_belt);
            }
        }

        // fill depth output belts
        let total_belts_to_adjust = belts_stack.len().saturating_sub(2);
        for (i, belt) in belts_stack.iter_mut().enumerate() {
            let _ = &mut self
                .output
                .context_handle(ContextLevel::Micro, "fill".into());
            belt.add_straight(total_belts_to_adjust.saturating_sub(i));
        }

        // padding
        // for belt in belts_stack.iter_mut() {
        //     belt.add_straight(3);
        // }

        // source belt keeeps passing through...
        let mut source_belt = belts_stack.remove(0);
        // source_belt.add_straight(5);

        // add skips
        for (output_belt_num, [first, last]) in belts_stack.iter_mut().array_chunks().enumerate() {
            let _ = &mut self
                .output
                .context_handle(ContextLevel::Micro, "ends".into());
            for skip_i in 0..(output_belt_num.div_floor(2)) {
                println!("skip_i {skip_i}");
                first.add_straight_underground(4);
                last.add_straight_underground(4);
            }

            if output_belt_num % 2 == 1 {
                first.add_straight_underground(1);
                last.add_straight_underground(1);
            }

            first.add_straight(1);

            last.add_straight_underground(1);
            last.add_turn90(!clockwise);
            last.add_turn90(!clockwise);
            last.add_straight(1);
        }

        for _ in 0..output_belts {
            let _ = &mut self
                .output
                .context_handle(ContextLevel::Micro, "tail".into());
            source_belt.add_straight_underground(1);
        }
    }
}
