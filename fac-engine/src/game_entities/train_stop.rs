use crate::{
    common::{
        entity::{FacEntity, SquareArea},
        names::FacEntityName,
    },
    def_entity_name,
};

use super::direction::{FacDirectionEighth, FacDirectionQuarter};

#[derive(Debug)]
pub struct FacEntTrainStop {
    direction: FacDirectionQuarter,
    station_name: String,
}

impl FacEntity for FacEntTrainStop {
    def_entity_name!(FacEntityName::TrainStop);

    fn to_fac_direction(&self) -> Option<FacDirectionEighth> {
        Some(self.direction.to_direction_eighth())
    }

    fn to_fac_station(&self) -> Option<String> {
        Some(self.station_name.clone())
    }
}

impl SquareArea for FacEntTrainStop {
    fn area_diameter() -> usize {
        2
    }
}

impl FacEntTrainStop {
    pub fn new(direction: FacDirectionQuarter, station_name: String) -> Self {
        Self {
            direction,
            station_name,
        }
    }
}
