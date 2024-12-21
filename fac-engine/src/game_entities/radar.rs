use crate::{
    common::{
        entity::{FacEntity, SquareArea},
        names::FacEntityName,
    },
    def_entity_name,
};

#[derive(Debug)]
pub struct FacEntRadar {}

impl FacEntity for FacEntRadar {
    def_entity_name!(FacEntityName::Radar);
}

impl SquareArea for FacEntRadar {
    fn area_diameter() -> usize {
        3
    }
}

impl FacEntRadar {
    pub fn new() -> Self {
        Self {}
    }
}
