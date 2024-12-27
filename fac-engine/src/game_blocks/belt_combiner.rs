use std::rc::Rc;

use crate::{
    blueprint::output::FacItemOutput,
    common::vpoint::VPoint,
    game_entities::{
        belt::FacEntBeltType, belt_split::FacEntBeltSplit, direction::FacDirectionQuarter,
    },
};

use super::{belt_bettel::FacBlkBettelBelt, block::FacBlock};

pub struct FacBlkBeltCombiner {
    input_belts: usize,
    belt: FacEntBeltType,
    layout: FacExtCombinerStage,
    direction: FacDirectionQuarter,
    output: Rc<FacItemOutput>,
}

pub enum FacExtCombinerStage {
    Fixed(usize),
}

impl FacBlock for FacBlkBeltCombiner {
    fn generate(&self, origin: VPoint) {
        self.generate_fixed(origin, 2);
    }
}

impl FacBlkBeltCombiner {
    fn generate_fixed(&self, origin: VPoint, stages: usize) {
        let mut belt =
            FacBlkBettelBelt::new(self.belt, origin, self.direction, self.output.clone());
        belt.add_straight(5);
        belt.add_turn90(true);

        FacEntBeltSplit::new(self.belt, self.direction);
    }
}
