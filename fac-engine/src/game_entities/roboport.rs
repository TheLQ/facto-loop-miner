use crate::{
    common::{
        entity::{FacEntity, SquareArea},
        names::FacEntityName,
    },
    def_entity_name,
};

pub const FACENT_ROBOPORT_DIAMETER: usize = 4;

#[derive(Debug)]
pub struct FacEntRoboport {}

impl FacEntity for FacEntRoboport {
    def_entity_name!(FacEntityName::Roboport);
}

impl SquareArea for FacEntRoboport {
    fn area_diameter() -> usize {
        FACENT_ROBOPORT_DIAMETER
    }
}

impl FacEntRoboport {
    pub fn new() -> Self {
        Self {}
    }
}
