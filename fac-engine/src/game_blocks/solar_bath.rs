use std::rc::Rc;

use crate::{
    admiral::generators::xy_grid_vpoint,
    blueprint::{bpitem::BlueprintItem, output::FacItemOutput},
    common::{
        entity::{FacEntity, SquareArea},
        vpoint::VPoint,
    },
    game_entities::{
        electric_large::{FacEntElectricLarge, FacEntElectricLargeType},
        lamp::FacEntLamp,
        solar::FacEntSolar,
    },
};

use super::block::FacBlock;

/// Take a bath in that sunshine
pub struct FacBlkSolarBath {
    pub width: usize,
    pub height: usize,
    pub output: Rc<FacItemOutput>,
}
impl FacBlock for FacBlkSolarBath {
    fn generate(&self, origin: VPoint) {
        for width in 0..self.width {
            for height in 0..self.height {
                let offset = FacEntSolar::area_diameter() * 5;
                self.place_solar_block(origin.move_xy_usize(offset * width, offset * height));
            }
        }
    }
}

impl FacBlkSolarBath {
    fn place_solar_block(&self, origin: VPoint) {
        for p in xy_grid_vpoint(origin, 5, 5, FacEntSolar::area_diameter() as u32) {
            if p.ix == 2 && p.iy == 2 {
                self.output.write(BlueprintItem::new(
                    FacEntElectricLarge::new(FacEntElectricLargeType::Substation).into_boxed(),
                    p.point(),
                ));
                self.output.write(BlueprintItem::new(
                    FacEntLamp::new().into_boxed(),
                    p.point().move_xy(2, 2),
                ));
            } else {
                self.output.write(BlueprintItem::new(
                    FacEntSolar::new().into_boxed(),
                    p.point(),
                ));
            }
        }
    }
}
