use crate::{
    common::{
        entity::{FacEntity, SquareArea, unwrap_options_to_option_vec},
        names::FacEntityName,
    },
    def_entity_name,
};

use super::{
    direction::{FacDirectionEighth, FacDirectionQuarter},
    module::FacModule,
};

pub const ELECTRIC_DRILL_SIZE: usize = 3;

#[derive(Debug)]
pub struct FacEntMiningDrillElectric {
    direction: FacDirectionQuarter,
    modules: [Option<FacModule>; 3],
}

impl FacEntity for FacEntMiningDrillElectric {
    def_entity_name!(FacEntityName::ElectricMiningDrill);

    fn to_fac_direction(&self) -> Option<FacDirectionEighth> {
        Some(self.direction.to_direction_eighth())
    }

    fn to_fac_items(&self) -> Option<Vec<FacModule>> {
        unwrap_options_to_option_vec(&self.modules)
    }
}

impl SquareArea for FacEntMiningDrillElectric {
    fn area_diameter() -> usize {
        ELECTRIC_DRILL_SIZE
    }
}

impl FacEntMiningDrillElectric {
    pub fn new(direction: FacDirectionQuarter) -> Self {
        Self {
            direction,
            modules: Default::default(),
        }
    }
}
