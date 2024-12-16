use crate::common::{entity::FacEntity, vpoint::VPoint};

use super::bpfac::entity::FacBpEntity;

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

    pub fn to_blueprint(&self) -> FacBpEntity {
        self.entity().to_fac(0, self.position())
    }
}
