use exhaustive::Exhaustive;

use crate::common::{
    entity::{FacEntity, SquareArea},
    names::FacEntityName,
};

pub const FACENT_ELECTRIC_LARGE_DIAMETER: usize = 2;

#[derive(Debug, Clone, Copy, PartialEq, Exhaustive)]
pub enum FacEntElectricLargeType {
    Substation,
    Big,
}

#[derive(Debug)]
pub struct FacEntElectricLarge {
    etype: FacEntElectricLargeType,
}

impl FacEntity for FacEntElectricLarge {
    fn name(&self) -> FacEntityName {
        FacEntityName::ElectricLarge(self.etype)
    }
}

impl SquareArea for FacEntElectricLarge {
    fn area_diameter() -> usize {
        FACENT_ELECTRIC_LARGE_DIAMETER
    }
}

impl FacEntElectricLarge {
    pub fn new(etype: FacEntElectricLargeType) -> Self {
        Self { etype }
    }
}
