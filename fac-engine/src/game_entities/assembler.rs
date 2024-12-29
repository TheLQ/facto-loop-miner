use super::{module::FacModule, tier::FacTier};
use crate::common::{
    entity::{FacEntity, SquareArea, unwrap_options_to_option_vec},
    names::FacEntityName,
};

pub type FacEntAssemblerModSlice = [Option<FacModule>; 3];

#[derive(Debug, Clone)]
pub struct FacEntAssembler {
    recipe: FacEntityName,
    tier: FacTier,
    modules: FacEntAssemblerModSlice,
}

impl SquareArea for FacEntAssembler {
    fn area_diameter() -> usize {
        3
    }
}

impl FacEntity for FacEntAssembler {
    fn name(&self) -> FacEntityName {
        FacEntityName::Assembler(self.tier)
    }

    fn to_fac_recipe(&self) -> Option<FacEntityName> {
        Some(self.recipe.clone())
    }

    fn to_fac_items(&self) -> Option<Vec<FacModule>> {
        unwrap_options_to_option_vec(&self.modules)
    }
}

impl FacEntAssembler {
    pub fn new(tier: FacTier, recipe: FacEntityName, modules: FacEntAssemblerModSlice) -> Self {
        Self {
            tier,
            recipe,
            modules,
        }
    }

    pub fn new_basic(level: FacTier, recipe: FacEntityName) -> Self {
        Self::new(level, recipe, [const { None }; 3])
    }

    pub fn recipe(&self) -> &FacEntityName {
        &self.recipe
    }
}
