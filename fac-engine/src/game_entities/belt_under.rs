use crate::common::{
    entity::{FacEntity, SquareArea},
    names::FacEntityName,
};

use super::{belt::FacEntBeltType, direction::FacDirectionQuarter};

pub struct FacEntBeltUnder {
    name: FacEntityName,
    direction: FacDirectionQuarter,
}

impl FacEntity for FacEntBeltUnder {
    fn name(&self) -> &FacEntityName {
        &self.name
    }
}

impl SquareArea for FacEntBeltUnder {
    fn area_diameter() -> usize {
        1
    }
}

impl FacEntBeltUnder {
    pub fn new(btype: FacEntBeltType, direction: FacDirectionQuarter) -> Self {
        Self {
            name: FacEntityName::BeltSplit(btype),
            direction,
        }
    }
}
