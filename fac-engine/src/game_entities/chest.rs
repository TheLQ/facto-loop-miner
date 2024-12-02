use crate::{
    common::{entity::FacEntity, names::FacEntityName},
    def_entity_size_square,
};

#[derive(Clone)]
pub enum FacChestType {
    Wood,
    Iron,
    Steel,
    Active,
    Passive,
    Storage,
    Buffer,
    Requestor,
}

pub struct FacChest {
    ctype: FacChestType,
    name: FacEntityName,
}

impl FacEntity for FacChest {
    def_entity_size_square!(2);

    fn name(&self) -> &FacEntityName {
        &self.name
    }
}

impl FacChest {
    pub fn new(ctype: FacChestType) -> Self {
        Self {
            name: FacEntityName::Chest(ctype.clone()),
            ctype,
        }
    }
}
