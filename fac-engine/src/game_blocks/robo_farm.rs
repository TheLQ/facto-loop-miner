use std::cell;

use crate::{
    admiral::generators::xy_grid_vpoint,
    blueprint::bpitem::BlueprintItem,
    common::{entity::FacEntity, vpoint::VPoint},
    game_entities::{
        electric_pole_big::{
            FACENT_ELECTRIC_LARGE_DIAMETER, FacEntElectricPoleBig, FacEntElectricPoleBigType,
        },
        lamp::FacEntLamp,
        roboport::{FACENT_ROBOPORT_DIAMETER, FacEntRoboport},
    },
};

use super::block::FacBlock;

pub struct FacBlkRobofarm {
    pub width: u32,
    pub height: u32,
    pub is_row_depth_full: bool,
}

impl FacBlock for FacBlkRobofarm {
    fn generate(&self, origin: VPoint) -> Vec<BlueprintItem> {
        let mut res = Vec::new();

        let cell_robo_width = if self.is_row_depth_full { 5 } else { 3 };

        for cell_pos in xy_grid_vpoint(
            origin,
            self.width,
            self.height,
            cell_robo_width * FACENT_ROBOPORT_DIAMETER as u32,
        ) {
            for pos in xy_grid_vpoint(
                cell_pos.point(),
                cell_robo_width,
                cell_robo_width,
                FACENT_ROBOPORT_DIAMETER as u32,
            ) {
                let center_i = cell_robo_width - /*center*/2 - /*count by 1*/1;
                if pos.ix == center_i && pos.iy == center_i {
                    // substation to grab all roboports
                    res.push(BlueprintItem::new(
                        FacEntElectricPoleBig::new(FacEntElectricPoleBigType::Substation)
                            .into_boxed(),
                        pos.point().move_xy(0, 1),
                    ));

                    // big pole to get all
                    res.push(BlueprintItem::new(
                        FacEntElectricPoleBig::new(FacEntElectricPoleBigType::Big).into_boxed(),
                        pos.point().move_xy(2, 1),
                    ));

                    // highlighter
                    res.push(BlueprintItem::new(
                        FacEntLamp::new().into_boxed(),
                        pos.point()
                            .move_xy(1, 1 + FACENT_ELECTRIC_LARGE_DIAMETER as i32),
                    ));
                } else {
                    res.push(BlueprintItem::new(
                        FacEntRoboport::new().into_boxed(),
                        pos.point(),
                    ));
                }
            }
        }

        res
    }
}