use crate::blueprint::bpitem::BlueprintItem;
use crate::common::vpoint::VPoint;
use crate::game_blocks::rail_hope::RailHopeAppender;
use crate::game_blocks::rail_hope_single::RailHopeSingle;
use crate::game_entities::direction::FacDirectionQuarter;
use crate::game_entities::rail::RAIL_STRAIGHT_DIAMETER;

// Side-by-side rail
pub struct RailHopeDual {
    hopes: [RailHopeSingle; 2],
}

impl RailHopeDual {
    pub fn new(origin: VPoint, origin_direction: FacDirectionQuarter) -> Self {
        let next_origin = origin.move_direction(
            origin_direction.rotate_opposite(),
            RAIL_STRAIGHT_DIAMETER * 2,
        );
        Self {
            hopes: [
                RailHopeSingle::new(origin, origin_direction.clone()),
                RailHopeSingle::new(next_origin, origin_direction),
            ],
        }
    }
}

impl RailHopeAppender for RailHopeDual {
    fn add_straight(&mut self, length: usize) {
        for rail in &mut self.hopes {
            rail.add_straight(length);
        }
    }

    fn add_turn90(&mut self, clockwise: bool) {
        if clockwise {
            self.hopes[1].add_straight(2);
        } else {
            self.hopes[0].add_straight(2);
        }

        self.hopes[0].add_turn90(clockwise);
        self.hopes[1].add_turn90(clockwise);

        if clockwise {
            self.hopes[1].add_straight(2);
        } else {
            self.hopes[0].add_straight(2);
        }
    }

    fn add_shift45(&mut self, _clockwise: bool, _length: usize) {
        unimplemented!()
    }

    fn to_fac(&self) -> Vec<BlueprintItem> {
        self.hopes.iter().flat_map(RailHopeSingle::to_fac).collect()
    }
}
