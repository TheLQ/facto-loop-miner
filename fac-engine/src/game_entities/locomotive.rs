use crate::{
    blueprint::bpfac::{position::FacBpPosition, schedule::FacBpSchedule},
    common::{
        entity::{FacArea, FacEntity, Size},
        names::FacEntityName,
        vpoint::VPoint,
    },
    def_entity_name,
};

use super::cargo_wagon::{rolling_stock_offset, rolling_stock_offset_from};

#[derive(Debug)]
pub struct FacEntLocomotive {
    schedule: Option<FacBpSchedule>,
}

impl FacEntity for FacEntLocomotive {
    def_entity_name!(FacEntityName::Locomotive);

    fn to_fac_schedule(&self) -> Option<FacBpSchedule> {
        self.schedule.clone()
    }
}

impl FacArea for FacEntLocomotive {
    fn rectangle_size(&self) -> Size {
        Size::rectangle(7, 2)
    }

    fn to_fac_position(&self, position: &VPoint) -> FacBpPosition {
        rolling_stock_offset(position)
    }

    fn from_fac_position(&self, position: &FacBpPosition) -> VPoint {
        rolling_stock_offset_from(position)
    }
}

impl FacEntLocomotive {
    pub fn new() -> Self {
        Self { schedule: None }
    }

    pub fn new_with_schedule(schedule: Option<FacBpSchedule>) -> Self {
        Self { schedule }
    }
}
