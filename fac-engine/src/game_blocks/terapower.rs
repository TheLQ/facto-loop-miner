use std::rc::Rc;

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
    pub output: Rc<FacItemOutput>,
}

impl FacBlock for FacBlkTerapower {
    fn generate(&self, origin: VPoint) {
        for pos in xy_grid_vpoint(origin, self.width, self.height, 30) {
            self.output.write(BlueprintItem::new(
                FacEntElectricLarge::new(FacEntElectricLargeType::Substation).into_boxed(),
                pos.point(),
            ));

            if pos.ix % 6 == 0 && pos.iy % 7 == 6 {
                self.output.write(BlueprintItem::new(
                    FacEntRadar::new().into_boxed(),
                    pos.point(),
                ));
            }
        }
    }
}

impl FacBlkTerapower {
    pub fn new(width: u32, height: u32, output: Rc<FacItemOutput>) -> Self {
        Self {
            width,
            height,
            output,
        }
    }
}
