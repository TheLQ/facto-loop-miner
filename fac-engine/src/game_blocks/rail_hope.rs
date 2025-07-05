use crate::common::vpoint::VPoint;
use crate::game_blocks::rail_hope_single::HopeLinkType;

pub trait RailHopeAppender {
    fn add_straight(&mut self, length: usize);

    fn add_straight_section(&mut self);

    fn add_turn90(&mut self, clockwise: bool);

    fn add_shift45(&mut self, clockwise: bool, length: usize);

    fn pos_next(&self) -> VPoint;
}

pub trait RailHopeLink {
    fn add_straight(&self, length: usize) -> Self;

    fn add_straight_section(&self) -> Self;

    fn add_turn90(&self, clockwise: bool) -> Self;

    fn add_shift45(&self, clockwise: bool, length: usize) -> Self;

    fn link_type(&self) -> HopeLinkType;

    fn pos_start(&self) -> VPoint;

    fn pos_next(&self) -> VPoint;

    fn area(&self, output: &mut Vec<VPoint>);

    fn area_vec(&self) -> Vec<VPoint> {
        let mut res = Vec::new();
        res.reserve(104);
        self.area(&mut res);
        if !matches!(res.len(), 104) {
            panic!("unknown {}", res.len());
        }
        res
    }
}
