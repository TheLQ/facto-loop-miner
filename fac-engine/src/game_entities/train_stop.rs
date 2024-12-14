use crate::{
    common::{
        entity::{FacEntity, SquareArea},
        names::FacEntityName,
    },
    def_entity_name,
};

use super::direction::FacDirectionQuarter;

pub struct FacTrainStop {
    direction: FacDirectionQuarter,
}

impl FacEntity for FacTrainStop {
    def_entity_name!(FacEntityName::TrainStop);
}

impl SquareArea for FacTrainStop {
    fn area_diameter() -> usize {
        1
    }
}

impl FacTrainStop {
    pub fn new(direction: FacDirectionQuarter) -> Self {
        Self { direction }
    }
}
