use crate::{
    common::{
        entity::{FacEntity, SquareArea, unwrap_options_to_option_vec},
        names::FacEntityName,
    },
    def_entity_name,
};

use super::module::FacModule;

#[derive(Debug)]
pub struct FacEntBeacon {
    modules: [Option<FacModule>; 2],
}

impl FacEntity for FacEntBeacon {
    def_entity_name!(FacEntityName::Beacon);

    fn to_fac_items(&self) -> Option<Vec<FacModule>> {
        unwrap_options_to_option_vec(&self.modules)
    }
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
