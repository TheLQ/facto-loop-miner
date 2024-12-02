use crate::{
    common::{entity::FacEntity, names::FacEntityName},
    def_entity_name, def_entity_size_square,
};

use super::direction::FacDirectionEighth;

pub struct FacRail {
    direction: FacDirectionEighth,
}

impl FacEntity for FacRail {
    def_entity_size_square!(2);
    def_entity_name!(FacEntityName::Rail);
}

impl FacRail {
    pub fn new(direction: FacDirectionEighth) -> Self {
        Self { direction }
    }
}
