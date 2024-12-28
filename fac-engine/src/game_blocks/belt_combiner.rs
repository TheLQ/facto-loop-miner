use std::rc::Rc;

use crate::{
    blueprint::output::FacItemOutput,
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
    Fixed(usize),
}

impl FacBlock for FacBlkBeltCombiner {
    fn generate(&self, origin: VPoint) {
        match &self.layout {
            FacExtCombinerStage::Fixed(stages) => self.generate_fixed(origin, *stages, true),
        };
    }
}

impl FacBlkBeltCombiner {
    fn generate_fixed(&self, origin: VPoint, stages: usize, clockwise: bool) {
        let mut belt =
            FacBlkBettelBelt::new(self.belt, origin, self.direction, self.output.clone());
        belt.add_straight(5);

        let mut belts_stack = vec![belt];
        for _stage in 0..stages {
            let cur_belt = belts_stack.last_mut().unwrap();
            cur_belt.add_split_priority(clockwise, FacEntBeltSplitPriority {
                input: FacExtPriority::None,
                output: FacExtPriority::Left,
            });

            let other_belt = cur_belt.belt_for_splitter();
            belts_stack.push(other_belt);
        }

        // let total_belts_to_adjust = belts_stack.len().saturating_sub(2);
        // for (i, belt) in belts_stack.iter_mut().enumerate() {
        //     belt.add_straight(total_belts_to_adjust.saturating_sub(i));
        // }

        // for belt in belts_stack.iter_mut() {
        //     belt.add_straight(5);
        // }
    }
}
