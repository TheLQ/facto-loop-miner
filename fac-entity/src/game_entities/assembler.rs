use crate::{
    common::{entity::FacEntity, names::FacEntityName},
    def_entity_size_square,
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

impl FacEntity for FacAssembler {
    def_entity_size_square!(3);

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
