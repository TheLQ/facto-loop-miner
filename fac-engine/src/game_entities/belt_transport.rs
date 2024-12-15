use crate::common::{
    entity::{FacEntity, SquareArea},
    names::FacEntityName,
};

use super::{belt::FacEntBeltType, direction::FacDirectionQuarter};

pub struct FacEntBeltTransport {
    name: FacEntityName,
    direction: FacDirectionQuarter,
}

impl FacEntity for FacEntBeltTransport {
    fn name(&self) -> &FacEntityName {
        &self.name
    }
}

impl SquareArea for FacEntBeltTransport {
    fn area_diameter() -> usize {
        1
    }
}

impl FacEntBeltTransport {
    pub fn new(btype: FacEntBeltType, direction: FacDirectionQuarter) -> Self {
        Self {
            name: FacEntityName::BeltTransport(btype),
            direction,
        }
    }
}
