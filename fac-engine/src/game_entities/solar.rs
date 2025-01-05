use crate::{
    common::{
        entity::{FacEntity, SquareArea},
        names::FacEntityName,
    },
    def_entity_name,
};

#[derive(Debug)]
pub struct FacEntSolar {}

impl FacEntity for FacEntSolar {
    def_entity_name!(FacEntityName::SolarPanel);
}

impl SquareArea for FacEntSolar {
    fn area_diameter() -> usize {
        3
    }
}

impl FacEntSolar {
    pub fn new() -> Self {
        Self {}
    }
}
