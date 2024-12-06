use crate::{
    common::{
        entity::{FacEntity, SquareArea},
        names::FacEntityName,
    },
    def_entity_name,
};

pub struct FacLamp {}

impl FacEntity for FacLamp {
    def_entity_name!(FacEntityName::Lamp);

    fn to_fac_name(&self) -> String {
        "small-lamp".into()
    }
}

impl SquareArea for FacLamp {
    fn area_diameter() -> usize {
        1
    }
}

impl FacLamp {
    pub fn new() -> Self {
        Self {}
    }
}
