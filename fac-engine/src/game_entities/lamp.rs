use crate::{
    common::{
        entity::{FacEntity, SquareArea},
        names::FacEntityName,
    },
    def_entity_name,
};

pub struct FacEntLamp {}

impl FacEntity for FacEntLamp {
    def_entity_name!(FacEntityName::Lamp);
}

impl SquareArea for FacEntLamp {
    fn area_diameter() -> usize {
        1
    }
}

impl FacEntLamp {
    pub fn new() -> Self {
        Self {}
    }
}
