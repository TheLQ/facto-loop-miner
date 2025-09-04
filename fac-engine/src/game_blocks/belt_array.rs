use crate::game_blocks::belt_bettel::FacBlkBettelBelt;
use itertools::Itertools;

pub struct FacBlkBettelArray {
    belts: Vec<FacBlkBettelBelt>,
}

impl FacBlkBettelArray {
    pub fn new(mut belts: Vec<FacBlkBettelBelt>) -> Self {
        let is_common_x = belts
            .iter()
            .map(|v| v.next_insert_position().x())
            .all_equal();
        let is_common_y = belts
            .iter()
            .map(|v| v.next_insert_position().y())
            .all_equal();

        match (is_common_x, is_common_y) {
            (true, false) => belts.sort_by_key(|left| left.next_insert_position().y()),
            (false, true) => belts.sort_by_key(|left| left.next_insert_position().x()),
            (true, true) => panic!("all the same?"),
            (false, false) => panic!("belts are not on axis"),
        }
        // uhh... the indexes must be reversed?

        Self { belts }
    }

    pub fn add_straight(&mut self, length: usize) {
        for belt in &mut self.belts {
            belt.add_straight(length)
        }
    }

    pub fn add_straight_underground(&mut self, length: usize) {
        for belt in &mut self.belts {
            belt.add_straight_underground(length)
        }
    }

    pub fn add_turn90_clk(&mut self) {
        let total = self.belts.len();
        for (i, belt) in self.belts.iter_mut().enumerate() {
            belt.add_turn90_stacked_row_clk(i, total);
        }
    }

    pub fn add_turn90_ccw(&mut self) {
        let total = self.belts.len();
        for (i, belt) in self.belts.iter_mut().enumerate() {
            belt.add_turn90_stacked_row_ccw(total - 1 - i)
        }
    }
}
