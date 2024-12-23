use crate::common::{
    entity::{FacEntity, SquareArea},
    names::FacEntityName,
};

use super::direction::{FacDirectionEighth, FacDirectionQuarter};

#[derive(Clone, Debug)]
pub enum FacEntRailSignalType {
    Basic,
    Chain,
}

#[derive(Debug)]
pub struct FacEntRailSignal {
    name: FacEntityName,
    direction: FacDirectionQuarter,
}

impl FacEntity for FacEntRailSignal {
    fn name(&self) -> &FacEntityName {
        &self.name
    }

    fn to_fac_direction(&self) -> Option<FacDirectionEighth> {
        Some(self.direction.to_direction_eighth())
    }
}

impl SquareArea for FacEntRailSignal {
    fn area_diameter() -> usize {
        1
    }
}

impl FacEntRailSignal {
    pub fn new(stype: FacEntRailSignalType, direction: FacDirectionQuarter) -> Self {
        Self {
            name: FacEntityName::RailSignal(stype),
            direction,
        }
    }
}
