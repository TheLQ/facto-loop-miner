use crate::common::{
    entity::{FacEntity, SquareArea},
    names::FacEntityName,
};

#[derive(Clone)]
pub enum FacEntityChestType {
    Wood,
    Iron,
    Steel,
    Active,
    Passive,
    Storage,
    Buffer,
    Requestor,
}

pub struct FacEntChest {
    name: FacEntityName,
}

impl FacEntity for FacEntChest {
    fn name(&self) -> &FacEntityName {
        &self.name
    }
}

impl SquareArea for FacEntChest {
    fn area_diameter() -> usize {
        1
    }
}

impl FacEntChest {
    pub fn new(ctype: FacEntityChestType) -> Self {
        Self {
            name: FacEntityName::Chest(ctype.clone()),
        }
    }

    pub fn chest_type(&self) -> &FacEntityChestType {
        match &self.name {
            FacEntityName::Chest(t) => t,
            _ => panic!("wtf"),
        }
    }
}
