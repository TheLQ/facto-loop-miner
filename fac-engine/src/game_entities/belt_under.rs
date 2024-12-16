use crate::common::{
    entity::{FacEntity, SquareArea},
    names::FacEntityName,
};

use super::{
    belt::FacEntBeltType,
    direction::{FacDirectionEighth, FacDirectionQuarter},
};

pub struct FacEntBeltUnder {
    name: FacEntityName,
    direction: FacDirectionQuarter,
}

impl FacEntity for FacEntBeltUnder {
    fn name(&self) -> &FacEntityName {
        &self.name
    }

    fn to_fac_direction(&self) -> Option<FacDirectionEighth> {
        Some(self.direction.to_direction_eighth())
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
            name: FacEntityName::BeltUnder(btype),
            direction,
        }
    }
}
