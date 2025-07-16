use super::direction::FacDirectionEighth;
use crate::{
    common::{
        entity::{FacEntity, SquareArea},
        names::FacEntityName,
    },
    def_entity_name,
};

pub const RAIL_STRAIGHT_DIAMETER: usize = 2;
pub const RAIL_STRAIGHT_DIAMETER_I32: i32 = 2;

#[derive(Debug)]
pub struct FacEntRailStraight {
    direction: FacDirectionEighth,
}

impl FacEntity for FacEntRailStraight {
    def_entity_name!(FacEntityName::RailStraight);

    fn to_fac_direction(&self) -> Option<FacDirectionEighth> {
        Some(self.direction)
    }
}

impl SquareArea for FacEntRailStraight {
    fn area_diameter() -> usize {
        RAIL_STRAIGHT_DIAMETER
    }
}

// impl FacArea for FacEntRailStraight {
//     fn rectangle_size(&self) -> Size {
//         Size::square(RAIL_STRAIGHT_DIAMETER)
//     }

//     fn to_fac_position(&self, position: &VPoint) -> FacBpPosition {
//         // position is exact because rail is complicated
//         position.to_fac_with_offset(0.0)
//     }
// }

impl FacEntRailStraight {
    pub fn new(direction: FacDirectionEighth) -> Self {
        Self { direction }
    }
}
