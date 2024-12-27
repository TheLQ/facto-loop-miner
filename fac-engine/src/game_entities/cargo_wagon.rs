use crate::{
    blueprint::bpfac::position::FacBpPosition,
    common::{
        entity::{FacArea, FacEntity, Size},
        names::FacEntityName,
        vpoint::VPoint,
    },
    def_entity_name,
};

#[derive(Debug)]
pub struct FacEntWagon {}

impl FacEntity for FacEntWagon {
    def_entity_name!(FacEntityName::CargoWagon);
}

impl FacArea for FacEntWagon {
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

impl FacEntWagon {
    pub fn new() -> Self {
        Self {}
    }
}

pub fn rolling_stock_offset(position: &VPoint) -> FacBpPosition {
    position.to_fac_with_offset_rectangle(7.0, 1.0)
}

pub fn rolling_stock_offset_from(position: &FacBpPosition) -> VPoint {
    position.to_vpoint_with_offset(7.0, 1.0)
}
