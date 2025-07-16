use super::block::FacBlock;
use crate::{
    admiral::generators::xy_grid_vpoint,
    blueprint::output::{ContextLevel, FacItemOutput},
    common::{entity::SquareArea, vpoint::VPoint},
    game_entities::{
        electric_large::FacEntElectricLargeType, lamp::FacEntLamp, solar::FacEntSolar,
    },
};
use std::rc::Rc;

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
                let _ = &mut self
                    .output
                    .context_handle(ContextLevel::Block, format!("SolarFarm-{width}-{height}"));
                let offset = FacEntSolar::area_diameter() * 5;
                self.place_solar_block(origin.move_xy_usize(offset * width, offset * height));
            }
        }
    }
}

impl FacBlkSolarBath {
    fn place_solar_block(&self, origin: VPoint) {
        for p in xy_grid_vpoint(origin, 5, 5, FacEntSolar::area_diameter() as u32) {
            let _ = &mut self
                .output
                .context_handle(ContextLevel::Micro, format!("Col-{}", p.ix));
            if p.ix == 2 && p.iy == 2 {
                self.output
                    .writei(FacEntElectricLargeType::Substation.entity(), p.point());
                self.output
                    .writei(FacEntLamp::new(), p.point().move_xy(2, 2));
            } else {
                self.output.writei(FacEntSolar::new(), p.point());
            }
        }
    }
}
