use exhaustive::Exhaustive;

use crate::common::{
    entity::{FacEntity, SquareArea},
    names::FacEntityName,
};

use super::direction::{FacDirectionEighth, FacDirectionQuarter};

#[derive(Debug, Clone, Copy, PartialEq, Exhaustive)]
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
    itype: FacEntInserterType,
    direction: FacDirectionQuarter,
}

impl FacEntity for FacEntInserter {
    fn name(&self) -> FacEntityName {
        FacEntityName::Inserter(self.itype)
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
        Self { itype, direction }
    }

    pub fn set_direction(&mut self, direction: FacDirectionQuarter) {
        self.direction = direction;
    }
}
