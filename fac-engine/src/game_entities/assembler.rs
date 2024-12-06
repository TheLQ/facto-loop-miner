use crate::common::{
    entity::{FacEntity, SquareArea},
    names::FacEntityName,
};

#[derive(Clone)]
pub enum FacAssemblerLevel {
    Tier1,
    Tier2,
    Tier3,
}

pub struct FacAssembler {
    level: FacAssemblerLevel,
    recipe: String,
    name: FacEntityName,
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

    fn to_fac_name(&self) -> String {
        match self.level {
            FacAssemblerLevel::Tier1 => "assembling-machine-1",
            FacAssemblerLevel::Tier2 => "assembling-machine-2",
            FacAssemblerLevel::Tier3 => "assembling-machine-3",
        }
        .into()
    }

    fn to_fac_recipe(&self) -> Option<String> {
        Some(self.recipe.clone())
    }
}

impl FacAssembler {
    pub fn new(level: FacAssemblerLevel, recipe: String) -> Self {
        Self {
            name: FacEntityName::Assembler(level.clone()),
            level,
            recipe,
        }
    }
}
