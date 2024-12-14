use crate::common::{
    entity::{FacEntity, SquareArea},
    names::FacEntityName,
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
    name: FacEntityName,
}

impl FacEntity for FacChest {
    fn name(&self) -> &FacEntityName {
        &self.name
    }
}

impl SquareArea for FacChest {
    fn area_diameter() -> usize {
        1
    }
}

impl FacChest {
    pub fn new(ctype: FacChestType) -> Self {
        Self {
            name: FacEntityName::Chest(ctype.clone()),
        }
    }

    pub fn chest_type(&self) -> &FacChestType {
        match &self.name {
            FacEntityName::Chest(t) => t,
            _ => panic!("wtf"),
        }
    }
}
