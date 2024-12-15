use crate::{
    common::{
        entity::{FacEntity, SquareArea},
        names::FacEntityName,
    },
    def_entity_name,
};

pub const FACENT_ELECTRIC_LARGE_DIAMETER: usize = 2;

pub enum FacEntElectricPoleBigType {
    Substation,
    Big,
}

pub struct FacEntElectricPoleBig {
    etype: FacEntElectricPoleBigType,
}

impl FacEntity for FacEntElectricPoleBig {
    def_entity_name!(FacEntityName::ElectricPoleBig);
}

impl SquareArea for FacEntElectricPoleBig {
    fn area_diameter() -> usize {
        FACENT_ELECTRIC_LARGE_DIAMETER
    }
}

impl FacEntElectricPoleBig {
    pub fn new(etype: FacEntElectricPoleBigType) -> Self {
        Self { etype }
    }
}
