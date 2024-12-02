use crate::{
    common::{entity::FacEntity, names::FacEntityName},
    def_entity_size_square,
};

pub enum FacInserterType {
    Burner,
    Basic,
    Long,
    Fast,
    Bulk,
}

pub struct FacInserter {
    typef: FacInserterType,
    // todo
    item: String,
    name: FacEntityName,
}

impl FacEntity for FacInserter {
    def_entity_size_square!(3);

    fn name(&self) -> &FacEntityName {
        &self.name
    }
}
