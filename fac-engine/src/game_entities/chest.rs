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
    ctype: FacChestType,
    name: FacEntityName,
}

impl FacEntity for FacChest {
    fn name(&self) -> &FacEntityName {
        &self.name
    }
    fn to_fac_name(&self) -> String {
        match self.ctype {
            FacChestType::Wood => "wooden-chest",
            FacChestType::Iron => "iron-chest",
            FacChestType::Steel => "steel-chest",
            FacChestType::Active => "logistic-chest-active-provider",
            FacChestType::Passive => "logistic-chest-passive-provider",
            FacChestType::Storage => "logistic-chest-storage",
            FacChestType::Buffer => "logistic-chest-buffer",
            FacChestType::Requestor => "logistic-chest-requestor",
        }
        .into()
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
            ctype,
        }
    }
}
