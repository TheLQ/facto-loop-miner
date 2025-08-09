use super::{
    direction::{FacDirectionEighth, FacDirectionQuarter},
    module::FacModule,
};
use crate::common::entity::SquareAreaConst;
use crate::{
    common::{
        entity::{FacEntity, unwrap_options_to_option_vec},
        names::FacEntityName,
    },
    def_entity_name, impl_square_area_const,
};

#[deprecated]
pub const ELECTRIC_DRILL_SIZE: usize = FacEntMiningDrillElectric::DIAMETER;

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

impl_square_area_const!(FacEntMiningDrillElectric, 3);

impl FacEntMiningDrillElectric {
    pub fn new(direction: FacDirectionQuarter) -> Self {
        Self::new_modules(direction, Default::default())
    }

    pub fn new_modules(direction: FacDirectionQuarter, modules: [Option<FacModule>; 3]) -> Self {
        Self { direction, modules }
    }
}
