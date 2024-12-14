use crate::{
    common::{
        entity::{FacEntity, SquareArea},
        names::FacEntityName,
    },
    def_entity_name,
};

use super::module::FacModule;

pub struct FacBeacon {
    modules: [Option<FacModule>; 2],
}

impl FacEntity for FacBeacon {
    def_entity_name!(FacEntityName::Beacon);

    fn to_fac_name(&self) -> String {
        "beacon".into()
    }
}

impl SquareArea for FacBeacon {
    fn area_diameter() -> usize {
        2
    }
}

impl FacBeacon {
    pub fn new(modules: [Option<FacModule>; 2]) -> Self {
        Self { modules }
    }
}
