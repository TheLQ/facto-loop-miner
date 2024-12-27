use std::rc::Rc;

use crate::{
    blueprint::output::{ContextLevel, FacItemOutput},
    common::vpoint::VPoint,
    game_entities::{belt::FacEntBeltType, direction::FacDirectionQuarter},
    util::bool_num::bool_to_num_usize,
};

use super::{belt_bettel::FacBlkBettelBelt, block::FacBlock};

pub struct FacBlkBeltTrainUnload {
    pub belt_type: FacEntBeltType,
    pub wagons: usize,
    pub output: Rc<FacItemOutput>,
    pub padding_unmerged: usize,
    pub padding_above: usize,
    pub padding_after: usize,
    pub turn_clockwise: bool,
    pub origin_direction: FacDirectionQuarter,
}

impl FacBlock for FacBlkBeltTrainUnload {
    fn generate(&self, origin: VPoint) {
        const BELTS_PER_WAGON: usize = 3;

        let _ = &mut self.output.context_handle(
            ContextLevel::Micro,
            format!("Unload-{}", self.origin_direction),
        );

        for wagon in 0..self.wagons {
            let origin = origin.move_direction_sideways_usize(self.origin_direction, wagon * 7);
            for output_belt in 0..BELTS_PER_WAGON {
                let turn_offset = if self.turn_clockwise {
                    BELTS_PER_WAGON - output_belt
                } else {
                    output_belt
                };
                let wagon_offset = if self.turn_clockwise {
                    (self.wagons * BELTS_PER_WAGON) - (wagon * BELTS_PER_WAGON)
                } else {
                    wagon * BELTS_PER_WAGON
                };
                let finish_straights = if self.turn_clockwise {
                    (self.padding_after + (BELTS_PER_WAGON * 2))
                        - (self.padding_after + /*our belts*/(output_belt * 2))
                } else {
                    self.padding_after + /*our belts*/(output_belt * 2)
                };

                let mut one_belt = self.add_dual_to_one(
                    origin.move_x_usize(2 * output_belt),
                    self.padding_unmerged,
                    self.padding_above + turn_offset + wagon_offset,
                );
                {
                    let _ = &mut self
                        .output
                        .context_handle(ContextLevel::Micro, format!("Finish-{finish_straights}"));
                    one_belt.add_turn90(self.turn_clockwise);
                    one_belt.add_straight(finish_straights);
                }
            }
        }
    }
}

impl FacBlkBeltTrainUnload {
    fn add_dual_to_one(
        &self,
        origin: VPoint,
        unmerged_height: usize,
        merged_height: usize,
    ) -> FacBlkBettelBelt {
        let _ = &mut self.output.context_handle(
            ContextLevel::Micro,
            format!("2to1-{unmerged_height}-{merged_height}"),
        );
        let mut belts = [
            FacBlkBettelBelt::new(
                self.belt_type.clone(),
                origin,
                self.origin_direction,
                self.output.clone(),
            ),
            FacBlkBettelBelt::new(
                self.belt_type.clone(),
                origin.move_direction(self.origin_direction, 1),
                self.origin_direction,
                self.output.clone(),
            ),
        ];

        for belt in belts.iter_mut() {
            belt.add_straight(unmerged_height - 1);
        }

        let clockwise = true;

        belts[bool_to_num_usize(!clockwise)].add_turn90(clockwise);
        belts[bool_to_num_usize(clockwise)].add_straight(1);

        let mut remaining_belt = {
            let [belt0, belt1] = belts;
            if clockwise { belt1 } else { belt0 }
        };
        remaining_belt.add_straight(merged_height - 1);
        remaining_belt
    }
}
