use crate::{
    admiral::generators::xy_grid_vpoint,
    blueprint::{bpitem::BlueprintItem, output::FacItemOutput},
    common::{entity::FacEntity, vpoint::VPoint},
    game_entities::{
        electric_large::{FacEntElectricLarge, FacEntElectricLargeType},
        radar::FacEntRadar,
    },
};

use super::block::FacBlock;

/// Max distance substation array
pub struct FacBlkTerapower {
    pub width: u32,
    pub height: u32,
}

impl FacBlock for FacBlkTerapower {
    fn generate(&self, origin: VPoint, output: &mut FacItemOutput) {
        for pos in xy_grid_vpoint(origin, self.width, self.height, 30) {
            output.write(BlueprintItem::new(
                FacEntElectricLarge::new(FacEntElectricLargeType::Substation).into_boxed(),
                pos.point(),
            ));

            if pos.ix % 6 == 0 && pos.iy % 7 == 6 {
                output.write(BlueprintItem::new(
                    FacEntRadar::new().into_boxed(),
                    pos.point(),
                ));
            }
        }
    }
}

impl FacBlkTerapower {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}
