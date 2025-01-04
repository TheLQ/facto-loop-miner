use exhaustive::Exhaustive;
use strum::AsRefStr;

use crate::common::{
    entity::{FacEntity, SquareArea},
    names::FacEntityName,
};

#[derive(Clone, Copy, Debug, PartialEq, AsRefStr, Exhaustive)]
pub enum FacEntConcreteType {
    Basic,
    Hazard,
    Refined,
    RefinedHazard,
}

#[derive(Debug)]
pub struct FacEntConcrete {
    ctype: FacEntConcreteType,
}

impl FacEntity for FacEntConcrete {
    fn name(&self) -> FacEntityName {
        FacEntityName::Concrete(self.ctype)
    }
}

impl SquareArea for FacEntConcrete {
    fn area_diameter() -> usize {
        1
    }
}

impl FacEntConcrete {
    pub fn new(ctype: FacEntConcreteType) -> Self {
        Self { ctype }
    }
}
