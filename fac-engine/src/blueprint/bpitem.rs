use crate::common::{entity::FacEntity, vpoint::VPoint};

pub struct BlueprintItem {
    entity: Box<dyn FacEntity>,
    position: VPoint,
}

impl BlueprintItem {
    pub fn new(entity: Box<dyn FacEntity>, position: VPoint) -> Self {
        Self { entity, position }
    }

    pub fn entity(&self) -> &Box<dyn FacEntity> {
        &self.entity
    }

    pub fn position(&self) -> &VPoint {
        &self.position
    }
}
