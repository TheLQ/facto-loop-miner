use crate::common::{
    entity::{FacEntity, SquareArea},
    names::FacEntityName,
};

#[derive(Clone)]

pub enum ElectricPoleSmallType {
    Wooden,
    Steel,
}

pub struct FacElectricPoleSmall {
    ptype: ElectricPoleSmallType,
    name: FacEntityName,
}

impl FacEntity for FacElectricPoleSmall {
    fn name(&self) -> &FacEntityName {
        &self.name
    }

    fn to_fac_name(&self) -> String {
        match self.ptype {
            ElectricPoleSmallType::Wooden => "small-electric-pole",
            ElectricPoleSmallType::Steel => "medium-electric-pole",
        }
        .into()
    }
}

impl SquareArea for FacElectricPoleSmall {
    fn area_diameter() -> usize {
        1
    }
}

impl FacElectricPoleSmall {
    pub fn new(ptype: ElectricPoleSmallType) -> Self {
        Self {
            name: FacEntityName::ElectricPoleSmall(ptype.clone()),
            ptype,
        }
    }
}
