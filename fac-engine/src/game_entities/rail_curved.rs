use crate::{
    blueprint::bpfac::position::FacBpPosition,
    common::{
        entity::{FacArea, FacEntity, Size},
        names::FacEntityName,
        vpoint::VPoint,
    },
    def_entity_name,
};

use super::direction::FacDirectionEighth;

#[derive(Debug)]
pub struct FacEntRailCurved {
    direction: FacDirectionEighth,
}

impl FacEntity for FacEntRailCurved {
    def_entity_name!(FacEntityName::RailCurved);

    fn to_fac_direction(&self) -> Option<FacDirectionEighth> {
        Some(self.direction)
    }
}

impl FacArea for FacEntRailCurved {
    fn rectangle_size(&self) -> Size {
        Size::square(8)
    }

    fn to_fac_position(&self, position: &VPoint) -> FacBpPosition {
        // position is exact because curved rail is complicated
        position.to_fac_exact()
    }

    fn from_fac_position(&self, position: &FacBpPosition) -> VPoint {
        position.to_vpoint_exact()
    }
}

impl FacEntRailCurved {
    pub fn new(direction: FacDirectionEighth) -> Self {
        Self { direction }
    }
}
