use crate::blueprint::bpitem::BlueprintItem;

pub trait RailHopeAppender {
    fn add_straight(&mut self, length: usize);

    fn add_turn90(&mut self, clockwise: bool);

    fn add_shift45(&mut self, clockwise: bool, length: usize);

    fn to_fac(&self) -> Vec<BlueprintItem>;
}
