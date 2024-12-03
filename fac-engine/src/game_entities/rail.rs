use crate::{
    common::{
        entity::{FacEntity, SquareArea},
        names::FacEntityName,
    },
    def_entity_name,
};

use super::direction::FacDirectionEighth;

pub struct FacRail {
    direction: FacDirectionEighth,
}

impl FacEntity for FacRail {
    def_entity_name!(FacEntityName::Rail);
}

impl SquareArea for FacRail {
    fn area_diameter() -> usize {
        1
    }
}

impl FacRail {
    pub fn new(direction: FacDirectionEighth) -> Self {
        Self { direction }
    }
}
