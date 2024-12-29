use crate::common::{
    entity::{FacEntity, SquareArea},
    names::FacEntityName,
};

use super::{
    belt::FacEntBeltType,
    direction::{FacDirectionEighth, FacDirectionQuarter},
};

#[derive(Debug)]
pub struct FacEntBeltTransport {
    btype: FacEntBeltType,
    direction: FacDirectionQuarter,
}

impl FacEntity for FacEntBeltTransport {
    fn name(&self) -> FacEntityName {
        FacEntityName::BeltTransport(self.btype)
    }

    fn to_fac_direction(&self) -> Option<FacDirectionEighth> {
        Some(self.direction.to_direction_eighth())
    }
}

impl SquareArea for FacEntBeltTransport {
    fn area_diameter() -> usize {
        1
    }
}

impl FacEntBeltTransport {
    pub fn new(btype: FacEntBeltType, direction: FacDirectionQuarter) -> Self {
        Self { btype, direction }
    }
}
