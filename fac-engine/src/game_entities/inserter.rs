use crate::common::{
    entity::{FacEntity, SquareArea},
    names::FacEntityName,
};

use super::direction::FacDirectionEighth;

#[derive(Clone)]
pub enum FacEntInserterType {
    Burner,
    Basic,
    Long,
    Fast,
    Filter,
    Stack,
    StackFilter,
}

pub struct FacEntInserter {
    name: FacEntityName,
    direction: FacDirectionEighth,
}

impl FacEntity for FacEntInserter {
    fn name(&self) -> &FacEntityName {
        &self.name
    }

    fn to_fac_direction(&self) -> Option<FacDirectionEighth> {
        Some(self.direction.clone())
    }
}
impl SquareArea for FacEntInserter {
    fn area_diameter() -> usize {
        1
    }
}

impl FacEntInserter {
    pub fn new(itype: FacEntInserterType, direction: FacDirectionEighth) -> Self {
        Self {
            name: FacEntityName::Inserter(itype),
            direction,
        }
    }

    pub fn inserter_type(&self) -> &FacEntInserterType {
        match &self.name {
            FacEntityName::Inserter(t) => t,
            _ => panic!("wtf"),
        }
    }
}
