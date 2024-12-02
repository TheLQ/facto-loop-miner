use crate::{
    common::{entity::FacEntity, names::FacEntityName},
    def_entity_name, def_entity_size_square,
};

pub struct FacLamp {}

impl FacEntity for FacLamp {
    def_entity_size_square!(1);
    def_entity_name!(FacEntityName::Lamp);
}

impl FacLamp {
    pub fn new() -> Self {
        Self {}
    }
}
