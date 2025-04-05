use std::rc::Rc;

use tracing::trace;

use crate::{
    blueprint::output::{ContextLevel, FacItemOutput},
    common::vpoint::VPoint,
    game_entities::{belt::FacEntBeltType, direction::FacDirectionQuarter},
    util::bool_num::bool_to_num_usize,
};

use super::belt_bettel::FacBlkBettelBelt;

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

// impl FacBlock for FacBlkBeltTrainUnload {
//     fn generate(&self, origin: VPoint) {}
// }

impl FacBlkBeltTrainUnload {
    pub fn generate(&self, origin: VPoint) -> Vec<FacBlkBettelBelt> {
        const DUALS_PER_WAGON: usize = 3;
        const BELTS_PER_DUAL: usize = 2;
        // const BELTS_PER_WAGON: usize = DUALS_PER_WAGON * BELTS_PER_DUAL;
        const WAGON_SIZE: usize = 7;

        let mut belts = Vec::new();
        for wagon in 0..self.wagons {
            let origin = origin.move_direction_sideways_usz(self.origin_direction, wagon * 7);
            for output_belt in 0..DUALS_PER_WAGON {
                let _ = &mut self.output.context_handle(
                    ContextLevel::Micro,
                    format!("Unload-{}-{wagon}-{output_belt}", self.origin_direction),
                );

                let turn_offset = if self.turn_clockwise {
                    DUALS_PER_WAGON - output_belt - 1
                } else {
                    output_belt
                };
                let wagon_offset = if self.turn_clockwise {
                    ((self.wagons - 1) * DUALS_PER_WAGON) - (wagon * DUALS_PER_WAGON)
                } else {
                    wagon * DUALS_PER_WAGON
                };

                let finish_straights = if self.turn_clockwise {
                    self.padding_after
                    // all belts
                    + ((DUALS_PER_WAGON - output_belt - 1) * BELTS_PER_DUAL)
                    // all wagons
                    + ((self.wagons - wagon - 1) * WAGON_SIZE)
                } else {
                    self.padding_after
                    // our belts
                    + (output_belt * BELTS_PER_DUAL)
                    // wagons
                    + (wagon * WAGON_SIZE)
                };

                let one_belt_origin =
                    origin.move_direction_sideways_usz(self.origin_direction, 2 * output_belt);
                trace!("one_belt from origin {origin} to {one_belt_origin}");
                let merged_height = self.padding_above + turn_offset + wagon_offset;
                let mut one_belt =
                    self.add_dual_to_one(one_belt_origin, self.padding_unmerged, merged_height);

                {
                    let _ = &mut self
                        .output
                        .context_handle(ContextLevel::Micro, format!("Finish-{finish_straights}"));
                    one_belt.add_turn90(self.turn_clockwise);
                    one_belt.add_straight(finish_straights);
                }
                belts.push(one_belt);
            }
        }
        belts
    }

    fn add_dual_to_one(
        &self,
        origin: VPoint,
        unmerged_height: usize,
        merged_height: usize,
    ) -> FacBlkBettelBelt {
        let _ = &mut self.output.context_handle(
            ContextLevel::Micro,
            format!("2to1-u{unmerged_height}-m{merged_height}"),
        );
        let mut belts = [
            FacBlkBettelBelt::new(
                self.belt_type,
                origin,
                self.origin_direction,
                self.output.clone(),
            ),
            FacBlkBettelBelt::new(
                self.belt_type,
                origin.move_direction_sideways_int(self.origin_direction, 1),
                self.origin_direction,
                self.output.clone(),
            ),
        ];

        for belt in belts.iter_mut() {
            belt.add_straight(unmerged_height + 1);
        }

        let clockwise = true;

        belts[bool_to_num_usize(!clockwise)].add_turn90(clockwise);
        belts[bool_to_num_usize(clockwise)].add_straight(1);

        let mut remaining_belt = {
            let [belt0, belt1] = belts;
            if clockwise { belt1 } else { belt0 }
        };
        remaining_belt.add_straight(merged_height);
        remaining_belt
    }
}
