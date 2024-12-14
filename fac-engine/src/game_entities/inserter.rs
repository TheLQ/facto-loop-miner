use crate::common::{
    entity::{FacEntity, SquareArea},
    names::FacEntityName,
};

use super::direction::FacDirectionEighth;

#[derive(Clone)]
pub enum FacInserterType {
    Burner,
    Basic,
    Long,
    Fast,
    Filter,
    Stack,
    StackFilter,
}

pub struct FacInserter {
    name: FacEntityName,
    direction: FacDirectionEighth,
}

impl FacEntity for FacInserter {
    fn name(&self) -> &FacEntityName {
        &self.name
    }

    fn to_fac_direction(&self) -> Option<FacDirectionEighth> {
        Some(self.direction.clone())
    }
}
impl SquareArea for FacInserter {
    fn area_diameter() -> usize {
        1
    }
}

impl FacInserter {
    pub fn new(itype: FacInserterType, direction: FacDirectionEighth) -> Self {
        Self {
            name: FacEntityName::Inserter(itype),
            direction,
        }
    }

    pub fn inserter_type(&self) -> &FacInserterType {
        match &self.name {
            FacEntityName::Inserter(t) => t,
            _ => panic!("wtf"),
        }
    }
}
