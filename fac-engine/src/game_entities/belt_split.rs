use crate::common::{
    entity::{FacArea, FacEntity, Size},
    names::FacEntityName,
};

use super::{
    belt::FacEntBeltType,
    direction::{FacDirectionEighth, FacDirectionQuarter},
};

#[derive(Debug)]
pub struct FacEntBeltSplit {
    name: FacEntityName,
    direction: FacDirectionQuarter,
}

impl FacEntity for FacEntBeltSplit {
    fn name(&self) -> &FacEntityName {
        &self.name
    }

    fn to_fac_direction(&self) -> Option<FacDirectionEighth> {
        Some(self.direction.to_direction_eighth())
    }
}

impl FacArea for FacEntBeltSplit {
    fn rectangle_size(&self) -> Size {
        match self.direction {
            FacDirectionQuarter::North | FacDirectionQuarter::South => Size::rectangle(2, 1),
            FacDirectionQuarter::East | FacDirectionQuarter::West => Size::rectangle(1, 2),
        }
    }
}

impl FacEntBeltSplit {
    pub fn new(btype: FacEntBeltType, direction: FacDirectionQuarter) -> Self {
        Self {
            name: FacEntityName::BeltSplit(btype),
            direction,
        }
    }
}
