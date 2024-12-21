use crate::{
    common::{
        entity::{FacEntity, SquareArea},
        names::FacEntityName,
    },
    def_entity_name,
};

#[derive(Debug)]
pub struct FacEntInfinityPower {}

impl FacEntity for FacEntInfinityPower {
    def_entity_name!(FacEntityName::InfinityPower);
}

impl SquareArea for FacEntInfinityPower {
    fn area_diameter() -> usize {
        2
    }
}

impl FacEntInfinityPower {
    pub fn new() -> Self {
        Self {}
    }
}
