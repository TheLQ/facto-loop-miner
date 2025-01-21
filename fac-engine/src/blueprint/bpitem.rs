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

    pub fn newb(entity: impl FacEntity + 'static, position: VPoint) -> Self {
        Self {
            entity: entity.into_boxed(),
            position,
        }
    }

    #[allow(clippy::borrowed_box)] // makes this not "object safe trait"
    pub fn entity(&self) -> &Box<dyn FacEntity> {
        &self.entity
    }

    pub fn position(&self) -> &VPoint {
        &self.position
    }

    pub fn to_blueprint(&self) -> FacBpEntity {
        self.entity().to_blueprint(None, self.position())
    }
}
