use crate::common::{
    entity::{FacEntity, SquareArea},
    names::FacEntityName,
};

pub const FACENT_ELECTRIC_LARGE_DIAMETER: usize = 2;

#[derive(Debug, Clone)]
pub enum FacEntElectricLargeType {
    Substation,
    Big,
}

#[derive(Debug)]
pub struct FacEntElectricLarge {
    name: FacEntityName,
}

impl FacEntity for FacEntElectricLarge {
    fn name(&self) -> &FacEntityName {
        &self.name
    }
}

impl SquareArea for FacEntElectricLarge {
    fn area_diameter() -> usize {
        FACENT_ELECTRIC_LARGE_DIAMETER
    }
}

impl FacEntElectricLarge {
    pub fn new(etype: FacEntElectricLargeType) -> Self {
        Self {
            name: FacEntityName::ElectricLarge(etype),
        }
    }
}
