use std::rc::Rc;

use crate::blueprint::bpitem::BlueprintItem;
use crate::blueprint::output::{ContextLevel, FacItemOutput};
use crate::common::entity::FacEntity;
use crate::common::vpoint::VPoint;
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
        // let output0 = &mut output_cell.borrow_mut();
        // let output1 =;
        Self {
            output: output.clone(),
            hopes: [
                RailHopeSingle::new(origin, origin_direction.clone(), output.clone()),
                RailHopeSingle::new(next_origin, origin_direction, output.clone()),
            ],
        }
    }

    pub fn add_straight_section(&mut self) {
        {
            let _ = &mut self
                .output
                .context_handle(ContextLevel::Micro, format!("👐Ruby"));
            self.add_electric_next();
        }

        for (i, rail) in &mut self.hopes.iter_mut().enumerate() {
            let _ = &mut self
                .output
                .context_handle(ContextLevel::Micro, format!("👐Dual-{}", i));
            rail.add_straight(15);
        }
    }

    pub fn add_electric_next(&mut self) {
        // todo: self.current_direction() causes
        // cannot borrow `self.electric_larges` as mutable because it is also borrowed as immutable
        let last_link = self.hopes[0].last_link();
        let cur_direction = last_link.next_direction;

        let electric_large_pos = last_link
            .start
            .move_direction_sideways_int(cur_direction, -2);
        self.output.write(BlueprintItem::new(
            FacEntElectricLarge::new(FacEntElectricLargeType::Big).into_boxed(),
            electric_large_pos,
        ));

        let lamp_pos = electric_large_pos.move_direction_usz(cur_direction, 1);
        self.output
            .write(BlueprintItem::new(FacEntLamp::new().into_boxed(), lamp_pos));
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
        for rail in &mut self.hopes {
            rail.add_straight(length);
        }
    }

    fn add_turn90(&mut self, clockwise: bool) {
        // let _ = &mut self
        //     .output
        //     .context_handle(ContextLevel::Micro, "👐Dual-Turn".into());
        self.add_electric_next();
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
    }

    fn add_shift45(&mut self, _clockwise: bool, _length: usize) {
        unimplemented!()
    }
}
