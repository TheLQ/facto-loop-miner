use std::rc::Rc;

use crate::blueprint::output::{ContextLevel, FacItemOutput};
use crate::common::vpoint::{VPOINT_ONE, VPoint};
use crate::game_blocks::rail_hope::RailHopeAppender;
use crate::game_blocks::rail_hope_single::RailHopeSingle;
use crate::game_entities::direction::FacDirectionQuarter;
use crate::game_entities::electric_large::{FacEntElectricLarge, FacEntElectricLargeType};
use crate::game_entities::lamp::FacEntLamp;
use crate::game_entities::rail_straight::RAIL_STRAIGHT_DIAMETER;

// Side-by-side rail
pub struct RailHopeDual {
    hopes: [RailHopeSingle; 2],
    output: Rc<FacItemOutput>,
}

impl RailHopeDual {
    pub fn new(
        origin: VPoint,
        origin_direction: FacDirectionQuarter,
        output: Rc<FacItemOutput>,
    ) -> Self {
        let next_origin = origin.move_direction_usz(
            origin_direction.rotate_opposite(),
            RAIL_STRAIGHT_DIAMETER * 2,
        );
        Self {
            output: output.clone(),
            hopes: [
                RailHopeSingle::new(origin, origin_direction, output.clone()),
                RailHopeSingle::new(next_origin, origin_direction, output.clone()),
            ],
        }
    }

    pub fn add_straight_section(&mut self) {
        self.add_straight(15);
        {
            let _ = &mut self
                .output
                .context_handle(ContextLevel::Micro, "👐Ruby".into());
            self.add_electric_next();
        }
    }

    pub fn add_electric_next(&mut self) {
        let last_link = self.hopes[0].appender_link();
        self.add_electric_next_for_link(
            last_link.next_direction,
            last_link.next_straight_position(),
        );
    }

    pub fn add_electric_next_for_link(&mut self, direction: FacDirectionQuarter, pos: VPoint) {
        // must use next pos, because last start link might be part of a turn90
        let electric_large_pos = pos.move_direction_sideways_int(direction, -2);
        self.output.writei(
            FacEntElectricLarge::new(FacEntElectricLargeType::Big),
            electric_large_pos,
        );

        self.output.writei(
            FacEntLamp::new(),
            (electric_large_pos + VPOINT_ONE).move_factorio_style_direction(direction, 1.5),
        );
    }

    pub(crate) fn next_buildable_point(&self) -> VPoint {
        self.hopes[0].next_pos()
    }

    pub(crate) fn current_direction(&self) -> &FacDirectionQuarter {
        &self.hopes[0].last_link().next_direction
    }
}

impl RailHopeAppender for RailHopeDual {
    fn add_straight(&mut self, length: usize) {
        for (i, rail) in &mut self.hopes.iter_mut().enumerate() {
            let _ = &mut self
                .output
                .context_handle(ContextLevel::Micro, format!("👐Dual-{}", i));
            rail.add_straight(length);
        }
    }

    fn add_turn90(&mut self, clockwise: bool) {
        // let _ = &mut self
        //     .output
        //     .context_handle(ContextLevel::Micro, "👐Dual-Turn".into());
        if clockwise {
            self.hopes[1].add_straight(2);
        } else {
            self.hopes[0].add_straight(2);
        }

        for rail in &mut self.hopes {
            rail.add_turn90(clockwise);
        }

        if clockwise {
            self.hopes[1].add_straight(2);
        } else {
            self.hopes[0].add_straight(2);
        }
        self.add_electric_next();
    }

    fn add_shift45(&mut self, _clockwise: bool, _length: usize) {
        unimplemented!()
    }
}
