use crate::common::{
    entity::{FacEntity, SquareArea},
    names::FacEntityName,
};

#[derive(Clone)]

pub enum ElectricPoleSmallType {
    Wooden,
    Steel,
}

pub struct FacEntElectricPoleSmall {
    name: FacEntityName,
}

impl FacEntity for FacEntElectricPoleSmall {
    fn name(&self) -> &FacEntityName {
        &self.name
    }
}

impl SquareArea for FacEntElectricPoleSmall {
    fn area_diameter() -> usize {
        1
    }
}

impl FacEntElectricPoleSmall {
    pub fn new(ptype: ElectricPoleSmallType) -> Self {
        Self {
            name: FacEntityName::ElectricPoleSmall(ptype),
        }
    }

    pub fn pole_type(&self) -> &ElectricPoleSmallType {
        match &self.name {
            FacEntityName::ElectricPoleSmall(t) => t,
            _ => panic!("wtf"),
        }
    }
}
