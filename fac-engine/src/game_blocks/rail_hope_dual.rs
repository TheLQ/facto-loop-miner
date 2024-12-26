use std::rc::Rc;

use crate::blueprint::bpitem::BlueprintItem;
use crate::blueprint::output::FacItemOutput;
use crate::common::entity::FacEntity;
use crate::common::vpoint::VPoint;
use crate::game_blocks::rail_hope::RailHopeAppender;
use crate::game_blocks::rail_hope_single::RailHopeSingle;
use crate::game_entities::direction::FacDirectionQuarter;
use crate::game_entities::electric_large::{FacEntElectricLarge, FacEntElectricLargeType};
use crate::game_entities::lamp::FacEntLamp;
use crate::game_entities::rail::RAIL_STRAIGHT_DIAMETER;

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
        let next_origin = origin.move_direction(
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
        self.add_electric_next();
        for rail in &mut self.hopes {
            rail.add_straight(15);
        }
    }

    pub fn add_electric_next(&mut self) {
        // todo: self.current_direction() causes
        // cannot borrow `self.electric_larges` as mutable because it is also borrowed as immutable
        let cur_direction = self.hopes[0].current_direction();

        let electric_large_pos = self.hopes[0]
            .current_next_pos()
            .move_direction_sideways(cur_direction, -2);
        self.output.write(BlueprintItem::new(
            FacEntElectricLarge::new(FacEntElectricLargeType::Big).into_boxed(),
            electric_large_pos,
        ));

        let lamp_pos = electric_large_pos.move_direction(cur_direction, 1);
        self.output
            .write(BlueprintItem::new(FacEntLamp::new().into_boxed(), lamp_pos));
    }

    pub(crate) fn next_buildable_point(&self) -> VPoint {
        self.hopes[0].current_next_pos()
    }

    pub(crate) fn current_direction(&self) -> &FacDirectionQuarter {
        self.hopes[0].current_direction()
    }
}

impl RailHopeAppender for RailHopeDual {
    fn add_straight(&mut self, length: usize) {
        for rail in &mut self.hopes {
            rail.add_straight(length);
        }
    }

    fn add_turn90(&mut self, clockwise: bool) {
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
