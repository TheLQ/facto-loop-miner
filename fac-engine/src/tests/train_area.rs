use crate::blueprint::output::FacItemOutput;
use crate::common::vpoint::VPOINT_TEN;
use crate::game_entities::direction::FacDirectionEighth;
use crate::game_entities::lamp::FacEntLamp;
use crate::game_entities::rail_curved::FacEntRailCurved;
use std::rc::Rc;

pub fn make_area_finder(output: Rc<FacItemOutput>) {
    output.writei(
        FacEntRailCurved::new(FacDirectionEighth::SouthEast),
        VPOINT_TEN,
    );

    output.writei(FacEntLamp::new(), VPOINT_TEN);
}
