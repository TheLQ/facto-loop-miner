use exhaustive::Exhaustive;

use crate::common::{
    entity::{FacEntity, SquareArea},
    names::FacEntityName,
};

use super::direction::{FacDirectionEighth, FacDirectionQuarter};

#[derive(Clone, Copy, Debug, PartialEq, Exhaustive)]
pub enum FacEntRailSignalType {
    Basic,
    Chain,
}

#[derive(Debug)]
pub struct FacEntRailSignal {
    stype: FacEntRailSignalType,
    direction: FacDirectionQuarter,
}

impl FacEntity for FacEntRailSignal {
    fn name(&self) -> FacEntityName {
        FacEntityName::RailSignal(self.stype)
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
        Self { stype, direction }
    }
}
