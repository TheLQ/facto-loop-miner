use std::rc::Rc;

use tracing::trace;

use super::belt_bettel::FacBlkBettelBelt;
use crate::common::vpoint_direction::VPointDirectionQ;
use crate::game_blocks::block::FacBlockFancy;
use crate::{
    blueprint::output::{ContextLevel, FacItemOutput},
    common::vpoint::VPoint,
    game_entities::belt::FacEntBeltType,
    util::bool_num::bool_to_num_usize,
};

pub struct FacBlkBeltTrainUnload {
    pub belt_type: FacEntBeltType,
    pub wagons: u32,
    pub output: Rc<FacItemOutput>,
    pub padding_unmerged: u32,
    pub padding_above: u32,
    pub padding_after: u32,
    pub turn_clockwise: bool,
    pub mode: UnloadMode,
    pub origin: VPointDirectionQ,
}

pub enum UnloadMode {
    Turn,
    Straight,
}

impl FacBlockFancy<Vec<FacBlkBettelBelt>> for FacBlkBeltTrainUnload {
    fn generate(&self) -> Vec<FacBlkBettelBelt> {
        match 1 {
            1 => self.generate_wrap_around_belts(),
            // 2 => self.generate_thru_belts(),
            _ => unimplemented!(),
        }
    }
}

pub const DUAL_BELTS_PER_WAGON: u32 = 3;
pub const BELTS_PER_DUAL: u32 = 2;
// const BELTS_PER_WAGON: usize = DUALS_PER_WAGON * BELTS_PER_DUAL;
// const WAGON_SIZE: u32 = 3;

impl FacBlkBeltTrainUnload {
    fn generate_wrap_around_belts(&self) -> Vec<FacBlkBettelBelt> {
        let mut belts = Vec::new();
        for wagon in 0..self.wagons {
            for (i, mut output_belt) in self.place_duals_for_wagon(wagon, 0).into_iter().enumerate()
            {
                let finish_straights = if self.turn_clockwise {
                    self.padding_after
                        // our belts
                        + ((DUAL_BELTS_PER_WAGON - (i as u32) - 1) * BELTS_PER_DUAL)
                        // belts per wagon
                        + ((self.wagons - wagon - 1) * DUAL_BELTS_PER_WAGON * BELTS_PER_DUAL)
                        // empty wagon separator
                        + (self.wagons - wagon - 1)
                } else {
                    self.padding_after
                        // our belts
                        + (i as u32 * BELTS_PER_DUAL)
                        // belts per wagon
                        + (wagon * DUAL_BELTS_PER_WAGON * BELTS_PER_DUAL)
                        // empty wagon separator
                        + wagon
                        // special always padding as this crosses
                        + 1
                };

                let _ = &mut self.output.context_handle(
                    ContextLevel::Micro,
                    format!("Finish-{finish_straights}-{wagon}"),
                );
                output_belt.add_turn90(self.turn_clockwise);
                match self.mode {
                    UnloadMode::Turn => {
                        output_belt.add_straight(finish_straights as usize);
                    }
                    UnloadMode::Straight => {
                        let remove_finished =
                            // turn point
                            i as u32
                            // wagon offset
                            + (wagon * DUAL_BELTS_PER_WAGON)
                            // remove special always padding
                            + if self.turn_clockwise { 0 } else { 1 };
                        let after_straights =
                            // wagon offset
                            ((self.wagons - wagon) * DUAL_BELTS_PER_WAGON)
                            - i as u32
                            - 1;
                        // remove special always padding
                        // + if self.turn_clockwise { 0 } else { 1 };
                        output_belt.add_straight(
                            (finish_straights.saturating_sub(remove_finished)) as usize,
                        );
                        output_belt.add_turn90(!self.turn_clockwise);
                        output_belt.add_straight(after_straights as usize);
                    }
                }

                belts.push(output_belt);
            }
        }
        belts
    }

    fn place_duals_for_wagon(&self, wagon: u32, one_belt_length: u32) -> Vec<FacBlkBettelBelt> {
        let mut res = Vec::new();
        let origin = self
            .origin
            .point()
            .move_direction_sideways_usz(self.origin.direction(), wagon as usize * 7);
        for output_belt in 0..DUAL_BELTS_PER_WAGON {
            let _ = &mut self.output.context_handle(
                ContextLevel::Micro,
                format!("Unload-{}-{wagon}-{output_belt}", self.origin.direction()),
            );

            let turn_offset = if self.turn_clockwise {
                DUAL_BELTS_PER_WAGON - output_belt - 1
            } else {
                output_belt
            };
            let wagon_offset = if self.turn_clockwise {
                ((self.wagons - 1) * DUAL_BELTS_PER_WAGON) - (wagon * DUAL_BELTS_PER_WAGON)
            } else {
                wagon * DUAL_BELTS_PER_WAGON
            };

            let one_belt_origin = origin
                .move_direction_sideways_usz(self.origin.direction(), (2 * output_belt) as usize);
            trace!("one_belt from origin {origin} to {one_belt_origin}");
            let merged_height = self.padding_above + turn_offset + wagon_offset;
            let one_belt =
                self.add_dual_to_one(one_belt_origin, self.padding_unmerged, merged_height);
            res.push(one_belt);
        }
        res
    }

    fn add_dual_to_one(
        &self,
        origin: VPoint,
        unmerged_height: u32,
        merged_height: u32,
    ) -> FacBlkBettelBelt {
        let _ = &mut self.output.context_handle(
            ContextLevel::Micro,
            format!("2to1-u{unmerged_height}-m{merged_height}"),
        );
        let mut belts = [
            FacBlkBettelBelt::new(
                self.belt_type,
                origin,
                *self.origin.direction(),
                self.output.clone(),
            ),
            FacBlkBettelBelt::new(
                self.belt_type,
                origin.move_direction_sideways_int(self.origin.direction(), 1),
                *self.origin.direction(),
                self.output.clone(),
            ),
        ];

        for belt in belts.iter_mut() {
            belt.add_straight(unmerged_height as usize + 1);
        }

        let clockwise = true;

        belts[bool_to_num_usize(!clockwise)].add_turn90(clockwise);
        belts[bool_to_num_usize(clockwise)].add_straight(1);

        let mut remaining_belt = {
            let [belt0, belt1] = belts;
            if clockwise { belt1 } else { belt0 }
        };
        remaining_belt.add_straight(merged_height as usize);
        remaining_belt
    }
}
