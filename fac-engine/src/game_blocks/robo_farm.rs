use std::rc::Rc;

use crate::{
    admiral::generators::xy_grid_vpoint,
    blueprint::{bpitem::BlueprintItem, output::FacItemOutput},
    common::{entity::FacEntity, vpoint::VPoint},
    game_entities::{
        electric_large::{
            FACENT_ELECTRIC_LARGE_DIAMETER, FacEntElectricLarge, FacEntElectricLargeType,
        },
        lamp::FacEntLamp,
        roboport::{FACENT_ROBOPORT_DIAMETER, FacEntRoboport},
    },
};

use super::block::FacBlock;

/// Large block of roboports
pub struct FacBlkRobofarm {
    pub width: u32,
    pub height: u32,
    pub is_row_depth_full: bool,
    pub output: Rc<FacItemOutput>,
}

impl FacBlock for FacBlkRobofarm {
    fn generate(&self, origin: VPoint) {
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
                    self.output.write(BlueprintItem::new(
                        FacEntElectricLarge::new(FacEntElectricLargeType::Substation).into_boxed(),
                        pos.point().move_xy(0, 1),
                    ));

                    // big pole to get all
                    self.output.write(BlueprintItem::new(
                        FacEntElectricLarge::new(FacEntElectricLargeType::Big).into_boxed(),
                        pos.point().move_xy(2, 1),
                    ));

                    // highlighter
                    self.output.write(BlueprintItem::new(
                        FacEntLamp::new().into_boxed(),
                        pos.point()
                            .move_xy_usize(1, 1 + FACENT_ELECTRIC_LARGE_DIAMETER),
                    ));
                } else {
                    self.output.write(BlueprintItem::new(
                        FacEntRoboport::new().into_boxed(),
                        pos.point(),
                    ));
                }
            }
        }
    }
}
