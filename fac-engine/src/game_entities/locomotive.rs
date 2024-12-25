use crate::{
    blueprint::bpfac::position::FacBpPosition,
    common::{
        entity::{FacArea, FacEntity, Size},
        names::FacEntityName,
        vpoint::VPoint,
    },
    def_entity_name,
};

use super::cargo_wagon::rolling_stock_offset;

#[derive(Debug)]
pub struct FacEntLocomotive {}

impl FacEntity for FacEntLocomotive {
    def_entity_name!(FacEntityName::Locomotive);
}

impl FacArea for FacEntLocomotive {
    fn rectangle_size(&self) -> Size {
        Size::rectangle(7, 2)
    }

    fn to_fac_position(&self, position: &VPoint) -> FacBpPosition {
        rolling_stock_offset(position)
    }
}

impl FacEntLocomotive {
    pub fn new() -> Self {
        Self {}
    }
}
