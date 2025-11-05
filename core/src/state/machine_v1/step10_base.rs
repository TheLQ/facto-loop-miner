use crate::state::err::XMachineResult;
use crate::state::machine::{Step, StepParams};
use crate::state::tuneables::BaseTunables;
use crate::surface::pixel::Pixel;
use crate::surfacev::vsurface::{
    VSurface, VSurfacePatchAsVs, VSurfacePatchAsVsMut, VSurfacePixelAsVsMut, VSurfacePixelMut,
};
use facto_loop_miner_fac_engine::common::varea::VArea;
use facto_loop_miner_fac_engine::common::vpoint::{VPOINT_ZERO, VPoint};
use itertools::Itertools;

pub struct Step10 {}

impl Step10 {
    pub fn new_boxed() -> Box<dyn Step> {
        Box::new(Step10 {})
    }
}

impl Step for Step10 {
    fn name(&self) -> &'static str {
        "step10-base"
    }

    fn transformer(&self, params: StepParams) -> XMachineResult<()> {
        let mut surface = VSurface::load_from_last_step(&params)?;
        let tunables = &surface.tunables().base.clone();

        // surface.remove_patches_within_radius(tunables.resource_clear_chunks.as_tiles_u32());
        surface
            .patches_mut()
            .remove_patches_in_column(tunables.resource_clear_chunks.as_tiles_u32());
        draw_mega_box(&mut surface.pixels_mut(), tunables);

        surface.save(&params.step_out_dir)?;
        Ok(())
    }
}

fn draw_mega_box(surface: &mut VSurfacePixelMut, tunables: &BaseTunables) {
    let base_tiles = tunables.base_chunks.as_tiles_u32();
    let box_points = points_in_centered_box(base_tiles, VPOINT_ZERO)
        .into_iter()
        .filter(|v| !v.is_within_center_radius(base_tiles - 50))
        .collect_vec();
    surface.change_pixels(box_points).stomp(Pixel::EdgeWall);
}

fn points_in_centered_box(radius: u32, center: VPoint) -> Vec<VPoint> {
    let offset = VPoint::new(radius as i32, radius as i32);
    let area = VArea::from_arbitrary_points_pair(center - offset, center + offset);
    area.get_points()
}
