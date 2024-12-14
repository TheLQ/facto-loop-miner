use super::{module::FacModule, tier::FacTier};
use crate::common::{
    entity::{FacEntity, SquareArea},
    names::FacEntityName,
};

pub struct FacAssembler {
    level: FacTier,
    recipe: String,
    name: FacEntityName,
    modules: [Option<FacModule>; 3],
}

impl SquareArea for FacAssembler {
    fn area_diameter() -> usize {
        3
    }
}

impl FacEntity for FacAssembler {
    fn name(&self) -> &FacEntityName {
        &self.name
    }

    fn to_fac_recipe(&self) -> Option<String> {
        Some(self.recipe.clone())
    }
}

impl FacAssembler {
    pub fn new(level: FacTier, recipe: String, modules: [Option<FacModule>; 3]) -> Self {
        Self {
            name: FacEntityName::Assembler(level.clone()),
            level,
            recipe,
            modules,
        }
    }

    pub fn new_basic(level: FacTier, recipe: String) -> Self {
        Self::new(level, recipe, [const { None }; 3])
    }
}
