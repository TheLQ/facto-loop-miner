use exhaustive::Exhaustive;

use crate::common::{
    entity::{FacEntity, SquareArea},
    names::FacEntityName,
};

#[derive(Clone, Copy, Debug, PartialEq, Exhaustive)]

pub enum FacEntElectricMiniType {
    Small,
    Medium,
}

#[derive(Debug)]
pub struct FacEntElectricMini {
    ptype: FacEntElectricMiniType,
}

impl FacEntity for FacEntElectricMini {
    fn name(&self) -> FacEntityName {
        FacEntityName::ElectricMini(self.ptype)
    }
}

impl SquareArea for FacEntElectricMini {
    fn area_diameter() -> usize {
        1
    }
}

impl FacEntElectricMini {
    pub fn new(ptype: FacEntElectricMiniType) -> Self {
        Self { ptype }
    }
}

impl FacEntElectricMiniType {
    pub fn entity(self) -> FacEntElectricMini {
        FacEntElectricMini::new(self)
    }
}
