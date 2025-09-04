use crate::game_blocks::belt_bettel::FacBlkBettelBelt;
use crate::game_entities::direction::FacDirectionQuarter;
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
        let is_reverse = matches!(
            self.belts[0].current_direction(),
            FacDirectionQuarter::South | FacDirectionQuarter::West
        );

        reversible_for_each(is_reverse, self.belts.iter_mut(), |(i, belt)| {
            belt.add_turn90_stacked_row_clk(i, total)
        });
    }

    pub fn add_turn90_ccw(&mut self) {
        let is_reverse = matches!(
            self.belts[0].current_direction(),
            FacDirectionQuarter::South | FacDirectionQuarter::West
        );

        reversible_for_each(is_reverse, self.belts.iter_mut(), |(i, belt)| {
            belt.add_turn90_stacked_row_ccw(i)
        });
    }
}

/// dumb workaround since generic arrays must be the same type
/// and the only alternative is Box<dyn Iterator>
fn reversible_for_each<T>(
    is_reverse: bool,
    arr: impl DoubleEndedIterator<Item = T>,
    cb: impl Fn((usize, T)),
) {
    if is_reverse {
        for v in arr.rev().enumerate() {
            cb(v)
        }
    } else {
        for v in arr.enumerate() {
            cb(v)
        }
    }
}
