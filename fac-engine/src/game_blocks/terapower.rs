use crate::{
    admiral::generators::{xy_grid, xy_grid_vpoint},
    blueprint::bpitem::BlueprintItem,
    common::{entity::FacEntity, vpoint::VPoint},
    game_entities::{electric_pole_big::FacElectricPoleBig, radar::FacRadar},
};

use super::block::BlockFac;

pub struct BlockFacTerapower {
    pub width: u32,
    pub height: u32,
}

impl BlockFac for BlockFacTerapower {
    fn generate(&self, origin: VPoint) -> Vec<BlueprintItem> {
        let mut res = Vec::new();
        for pos in xy_grid_vpoint(origin, self.width, self.height, 30) {
            res.push(BlueprintItem::new(
                FacElectricPoleBig::new().into_boxed(),
                pos.to_vpoint(),
            ));

            if pos.ix % 6 == 0 && pos.iy % 7 == 6 {
                res.push(BlueprintItem::new(
                    FacRadar::new().into_boxed(),
                    pos.to_vpoint(),
                ));
            }
        }
        res
    }
}
