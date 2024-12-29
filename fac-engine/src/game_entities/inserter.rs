use exhaustive::Exhaustive;

use crate::common::{
    entity::{FacEntity, SquareArea},
    names::FacEntityName,
};

use super::direction::{FacDirectionEighth, FacDirectionQuarter};

#[derive(Debug, Clone, PartialEq, Exhaustive)]
pub enum FacEntInserterType {
    Burner,
    Basic,
    Long,
    Fast,
    Filter,
    Stack,
    StackFilter,
}

#[derive(Debug, Clone)]
pub struct FacEntInserter {
    name: FacEntityName,
    direction: FacDirectionQuarter,
}

impl FacEntity for FacEntInserter {
    fn name(&self) -> &FacEntityName {
        &self.name
    }

    fn to_fac_direction(&self) -> Option<FacDirectionEighth> {
        Some(self.direction.to_direction_eighth())
    }
}
impl SquareArea for FacEntInserter {
    fn area_diameter() -> usize {
        1
    }
}

impl FacEntInserter {
    pub fn new(itype: FacEntInserterType, direction: FacDirectionQuarter) -> Self {
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

    pub fn set_direction(&mut self, direction: FacDirectionQuarter) {
        self.direction = direction;
    }
}
