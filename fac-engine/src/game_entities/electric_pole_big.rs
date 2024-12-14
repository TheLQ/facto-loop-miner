use crate::{
    common::{
        entity::{FacEntity, SquareArea},
        names::FacEntityName,
    },
    def_entity_name,
};

pub struct FacEntElectricPoleBig {}

impl FacEntity for FacEntElectricPoleBig {
    def_entity_name!(FacEntityName::ElectricPoleBig);
}

impl SquareArea for FacEntElectricPoleBig {
    fn area_diameter() -> usize {
        2
    }
}

impl FacEntElectricPoleBig {
    pub fn new() -> Self {
        Self {}
    }
}
