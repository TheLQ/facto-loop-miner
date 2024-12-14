use crate::{
    common::{
        entity::{FacEntity, SquareArea},
        names::FacEntityName,
    },
    def_entity_name,
};

pub struct FacRadar {}

impl FacEntity for FacRadar {
    def_entity_name!(FacEntityName::Radar);
}

impl SquareArea for FacRadar {
    fn area_diameter() -> usize {
        3
    }
}

impl FacRadar {
    pub fn new() -> Self {
        Self {}
    }
}
