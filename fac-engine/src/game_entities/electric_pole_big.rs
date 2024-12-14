use crate::{
    common::{
        entity::{FacEntity, SquareArea},
        names::FacEntityName,
    },
    def_entity_name,
};

pub struct FacElectricPoleBig {}

impl FacEntity for FacElectricPoleBig {
    def_entity_name!(FacEntityName::ElectricPoleBig);
}

impl SquareArea for FacElectricPoleBig {
    fn area_diameter() -> usize {
        2
    }
}

impl FacElectricPoleBig {
    pub fn new() -> Self {
        Self {}
    }
}
