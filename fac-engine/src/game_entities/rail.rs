use super::direction::FacDirectionEighth;
use crate::{
    blueprint::bpfac::position::FacBpPosition,
    common::{
        entity::{FacArea, FacEntity, Size},
        names::FacEntityName,
        vpoint::VPoint,
    },
    def_entity_name,
};

pub const RAIL_STRAIGHT_DIAMETER: usize = 2;

#[derive(Clone)]
pub enum FacEntRailType {
    Straight,
    Curved,
}

pub struct FacEntRail {
    rtype: FacEntRailType,
    direction: FacDirectionEighth,
}

impl FacEntity for FacEntRail {
    def_entity_name!(FacEntityName::Rail);
}

impl FacArea for FacEntRail {
    fn rectangle_size(&self) -> Size {
        match (&self.rtype, &self.direction) {
            (FacEntRailType::Straight, _) => Size::square(RAIL_STRAIGHT_DIAMETER),
            (FacEntRailType::Curved, _) => Size::square(8),
            // (
            //     FacEntRailType::Curved,
            //     FacDirectionEighth::North
            //     | FacDirectionEighth::NorthEast
            //     | FacDirectionEighth::South
            //     | FacDirectionEighth::SouthWest,
            // ) => Size::rectangle(5, 8),
            // (
            //     FacEntRailType::Curved,
            //     FacDirectionEighth::NorthWest
            //     | FacDirectionEighth::West
            //     | FacDirectionEighth::SouthEast
            //     | FacDirectionEighth::East,
            // ) => Size::rectangle(8, 5),
        }
    }

    fn to_fac_position(&self, position: &VPoint) -> FacBpPosition {
        todo!()
    }
}

impl FacEntRail {
    pub fn new(rtype: FacEntRailType, direction: FacDirectionEighth) -> Self {
        Self { rtype, direction }
    }
}
