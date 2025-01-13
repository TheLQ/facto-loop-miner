use crate::common::{
    entity::{FacEntity, SquareArea},
    names::FacEntityName,
};

#[derive(Debug)]
pub enum FacEntResourceType {
    IronOre,
    CopperOre,
}

impl FacEntResourceType {
    const fn to_fac_name(&self) -> FacEntityName {
        match self {
            Self::IronOre => FacEntityName::IronOre,
            Self::CopperOre => FacEntityName::CopperOre,
        }
    }

    pub fn entity(self) -> FacEntResource {
        FacEntResource::new(self)
    }
}

#[derive(Debug)]
pub struct FacEntResource {
    rtype: FacEntResourceType,
}

impl FacEntity for FacEntResource {
    fn name(&self) -> FacEntityName {
        self.rtype.to_fac_name()
    }
}

impl SquareArea for FacEntResource {
    fn area_diameter() -> usize {
        1
    }
}

impl FacEntResource {
    pub fn new(rtype: FacEntResourceType) -> Self {
        Self { rtype }
    }
}
