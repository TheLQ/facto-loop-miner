use crate::{
    admiral::generators::xy_grid_vpoint,
    blueprint::bpitem::BlueprintItem,
    common::{entity::FacEntity, vpoint::VPoint},
    game_entities::{beacon::FacEntBeacon, module::FacModule},
};

use super::block::FacBlock;

pub struct FacBlkBeaconFarm {
    pub inner_cell_size: u32,
    pub width: u32,
    pub height: u32,
    pub module: FacModule,
}

impl FacBlock for FacBlkBeaconFarm {
    fn generate(&self, origin: VPoint) -> Vec<BlueprintItem> {
        let mut res = Vec::new();
        let zero_cell_size = self.inner_cell_size + 1;

        for pos in xy_grid_vpoint(
            origin,
            (self.width * zero_cell_size) + 1,
            (self.height * zero_cell_size) + 1,
            3,
        ) {
            if pos.ix % zero_cell_size == 0 || pos.iy % zero_cell_size == 0 {
                res.push(BlueprintItem::new(
                    FacEntBeacon::new([Some(self.module.clone()), Some(self.module.clone())])
                        .into_boxed(),
                    pos.point(),
                ));
            }
        }

        res
    }
}
