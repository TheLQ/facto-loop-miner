use crate::common::{
    entity::{FacEntity, SquareArea},
    names::FacEntityName,
};

#[derive(Debug, Clone)]
pub enum FacEntChestType {
    Wood,
    Iron,
    Steel,
    Active,
    Passive,
    Storage,
    Buffer,
    Requestor,
}

#[derive(Debug, Clone)]
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
    pub fn new(ctype: FacEntChestType) -> Self {
        Self {
            name: FacEntityName::Chest(ctype.clone()),
        }
    }

    pub fn chest_type(&self) -> &FacEntChestType {
        match &self.name {
            FacEntityName::Chest(t) => t,
            _ => panic!("wtf"),
        }
    }
}
