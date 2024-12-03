use crate::common::{
    entity::{FacEntity, SquareArea},
    names::FacEntityName,
};

#[derive(Clone)]
pub enum FacInserterType {
    Burner,
    Basic,
    Long,
    Fast,
    Bulk,
}

pub struct FacInserter {
    itype: FacInserterType,
    name: FacEntityName,
}

impl FacEntity for FacInserter {
    fn name(&self) -> &FacEntityName {
        &self.name
    }
}
impl SquareArea for FacInserter {
    fn area_diameter() -> usize {
        1
    }
}

impl FacInserter {
    pub fn new(itype: FacInserterType) -> Self {
        Self {
            name: FacEntityName::Inserter(itype.clone()),
            itype,
        }
    }
}
