use crate::{
    common::{entity::FacEntity, names::FacEntityName},
    def_entity_size_square,
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
    def_entity_size_square!(3);

    fn name(&self) -> &FacEntityName {
        &self.name
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
