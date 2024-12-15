use crate::{
    common::{
        entity::{FacEntity, SquareArea},
        names::FacEntityName,
    },
    def_entity_name,
};

#[derive(Clone)]
pub enum FacEntBeltTransportType {
    Basic,
    Fast,
    Express,
}

pub struct FacEntBeltTransport {}

impl FacEntity for FacEntBeltTransport {
    def_entity_name!(FacEntityName::Lamp);
}

impl SquareArea for FacEntBeltTransport {
    fn area_diameter() -> usize {
        1
    }
}

impl FacEntBeltTransport {
    pub fn new() -> Self {
        Self {}
    }
}
