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
    Filter,
    Stack,
    StackFilter,
}

pub struct FacInserter {
    itype: FacInserterType,
    name: FacEntityName,
}

impl FacEntity for FacInserter {
    fn name(&self) -> &FacEntityName {
        &self.name
    }

    fn to_facto_name(&self) -> String {
        match self.itype {
            FacInserterType::Burner => "burner-inserter",
            FacInserterType::Basic => "inserter",
            FacInserterType::Long => "long-handed-inserter",
            FacInserterType::Fast => "fast-inserter",
            FacInserterType::Filter => "filter-inserter",
            FacInserterType::Stack => "stack-inserter",
            FacInserterType::StackFilter => "stack-filter-inserter",
        }
        .into()
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
