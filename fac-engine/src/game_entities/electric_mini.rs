use exhaustive::Exhaustive;

use crate::common::{
    entity::{FacEntity, SquareArea},
    names::FacEntityName,
};

#[derive(Clone, Debug, PartialEq, Exhaustive)]

pub enum FacEntElectricMiniType {
    Small,
    Medium,
}

#[derive(Debug)]
pub struct FacEntElectricMini {
    name: FacEntityName,
}

impl FacEntity for FacEntElectricMini {
    fn name(&self) -> &FacEntityName {
        &self.name
    }
}

impl SquareArea for FacEntElectricMini {
    fn area_diameter() -> usize {
        1
    }
}

impl FacEntElectricMini {
    pub fn new(ptype: FacEntElectricMiniType) -> Self {
        Self {
            name: FacEntityName::ElectricMini(ptype),
        }
    }

    pub fn pole_type(&self) -> &FacEntElectricMiniType {
        match &self.name {
            FacEntityName::ElectricMini(t) => t,
            _ => panic!("wtf"),
        }
    }
}
