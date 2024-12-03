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
    // todo
    item: String,
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
}

impl FacAssembler {
    pub fn new(level: FacAssemblerLevel, item: String) -> Self {
        Self {
            name: FacEntityName::Assembler(level.clone()),
            level,
            item,
        }
    }
}
