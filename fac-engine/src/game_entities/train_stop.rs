use crate::{
    common::{
        entity::{FacEntity, SquareArea},
        names::FacEntityName,
    },
    def_entity_name,
};

use super::direction::FacDirectionQuarter;

pub struct FacEntTrainStop {
    direction: FacDirectionQuarter,
}

impl FacEntity for FacEntTrainStop {
    def_entity_name!(FacEntityName::TrainStop);
}

impl SquareArea for FacEntTrainStop {
    fn area_diameter() -> usize {
        1
    }
}

impl FacEntTrainStop {
    pub fn new(direction: FacDirectionQuarter) -> Self {
        Self { direction }
    }
}
