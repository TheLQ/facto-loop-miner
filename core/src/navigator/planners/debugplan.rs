use crate::navigator::mine_permutate::get_possible_routes_for_batch;
use crate::navigator::mine_selector::{MineSelectBatch, select_mines_and_sources};
use crate::navigator::planners::PathingTunables;
use crate::navigator::planners::common::{debug_draw_complete_plan, draw_prep};
use crate::surfacev::vsurface::{
    AsVsPixel, AsVsPixelMut, VSurface, VSurfacePixel, VSurfacePixelMut, VSurfacePixelPatches,
    VSurfacePixelPatchesMut,
};
use facto_loop_miner_fac_engine::common::varea::VArea;
use simd_json::prelude::ArrayTrait;
use tracing::{info, trace};

pub fn start_debug_planner(tunables: &PathingTunables, surface_mut: &mut VSurfacePixelPatchesMut) {
    let select_batches = get_batches(tunables, surface_mut.pixel_patches());
    paint_result(&mut surface_mut.pixels_mut(), select_batches);
    // if let Err(()) = debug_conflict_no_touching(surface, &select_batches) {
    //     error!("no touching");
    //     return;
    // } else {
    //     error!("good touching");
    // }
}

fn get_batches(tunables: &PathingTunables, surface: VSurfacePixelPatches) -> Vec<MineSelectBatch> {
    let select_batches = select_mines_and_sources(tunables, surface, 5)
        .into_success()
        .unwrap();
    let mines: usize = select_batches
        .iter()
        .flat_map(|v| &v.mines)
        .map(|v| VSurfacePixelPatches::mine_patches_len(v))
        .sum();
    info!(
        "selected {mines} total patches in {} batches",
        select_batches.len()
    );

    let max_area = VArea::from_arbitrary_points(
        select_batches
            .iter()
            .flat_map(|v| &v.mines)
            .flat_map(|v| v.area_min().get_corner_points()),
    );
    let mut total_in_area = 0;
    for patch in surface.patches() {
        if max_area.contains_point(&patch.area.point_center()) {
            total_in_area += 1;
        }
    }
    info!("witihin area {max_area} is {total_in_area} patches");
    select_batches
}

fn paint_result(surface_mut: &mut VSurfacePixelMut, select_batches: Vec<MineSelectBatch>) {
    draw_prep(surface_mut, &select_batches);
    for (i, batch) in select_batches.into_iter().enumerate() {
        trace!("batch {i}");
        let plan = get_possible_routes_for_batch(surface_mut.pixels(), batch);
        debug_draw_complete_plan(surface_mut, plan);
    }
    surface_mut
        .pixels()
        .paint_pixel_colored_zoomed()
        .save_to_oculante();
}
