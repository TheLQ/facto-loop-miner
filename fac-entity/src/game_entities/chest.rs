use crate::{
    common::{entity::FacEntity, names::FacEntityName},
    def_entity_name, def_entity_size_square,
};

pub enum FacAssemblerLevel {
    Tier1,
    Tier2,
    Tier3,
}

pub struct FacChest {
    level: FacAssemblerLevel,
    // todo
    item: String,
}

impl FacEntity for FacChest {
    def_entity_size_square!(2);
    def_entity_name!(FacEntityName::Chest);
}
