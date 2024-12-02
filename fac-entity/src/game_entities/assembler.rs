use crate::{
    common::{entity::FacEntity, names::FacEntityName},
    def_entity_size_square,
};

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
