use crate::{
    admiral::generators::xy_grid_vpoint,
    blueprint::{bpitem::BlueprintItem, output::FacItemOutput},
    common::{entity::FacEntity, vpoint::VPoint},
    game_entities::{beacon::FacEntBeacon, module::FacModule},
};

use super::block::FacBlock;

/// Beacon farm to support assemblers or smelters
pub struct FacBlkBeaconFarm<C: FacBlock> {
    pub inner_cell_size: u32,
    pub width: u32,
    pub height: u32,
    pub module: FacModule,
    pub cell: Option<C>,
}

impl<C: FacBlock> FacBlock for FacBlkBeaconFarm<C> {
    fn generate(&self, origin: VPoint, output: &mut FacItemOutput) {
        let zero_cell_size = self.inner_cell_size + 1;

        for pos in xy_grid_vpoint(
            origin,
            (self.width * zero_cell_size) + 1,
            (self.height * zero_cell_size) + 1,
            3,
        ) {
            if pos.ix % zero_cell_size == 0 || pos.iy % zero_cell_size == 0 {
                output.write(BlueprintItem::new(
                    FacEntBeacon::new([Some(self.module.clone()), Some(self.module.clone())])
                        .into_boxed(),
                    pos.point(),
                ));
            } else if pos.ix % zero_cell_size == 1 && pos.iy % zero_cell_size == 1 {
                if let Some(inner) = &self.cell {
                    inner.generate(pos.point(), output);
                }
            }
        }
    }
}
