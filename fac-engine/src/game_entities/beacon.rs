use crate::{
    common::{
        entity::{FacEntity, SquareArea},
        names::FacEntityName,
    },
    def_entity_name,
};

use super::module::FacModule;

pub struct FacEntBeacon {
    modules: [Option<FacModule>; 2],
}

impl FacEntity for FacEntBeacon {
    def_entity_name!(FacEntityName::Beacon);
}

impl SquareArea for FacEntBeacon {
    fn area_diameter() -> usize {
        3
    }
}

impl FacEntBeacon {
    pub fn new(modules: [Option<FacModule>; 2]) -> Self {
        Self { modules }
    }
}
