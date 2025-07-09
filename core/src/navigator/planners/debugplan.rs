use crate::navigator::mine_permutate::get_possible_routes_for_batch;
use crate::navigator::mine_selector::select_mines_and_sources;
use crate::navigator::planners::common::{debug_draw_complete_plan, draw_prep};
use crate::surfacev::err::VResult;
use crate::surfacev::vsurface::VSurface;
use facto_loop_miner_fac_engine::common::varea::VArea;
use itertools::Itertools;
use simd_json::prelude::ArrayTrait;
use tracing::{info, trace};

pub fn start_debug_planner(surface: &mut VSurface) -> VResult<()> {
    let select_batches = select_mines_and_sources(&surface, 5)
        .into_success()
        .unwrap();
    let mines: usize = select_batches
        .iter()
        .flat_map(|v| &v.mines)
        .map(|v| v.surface_patches_len())
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
    for patch in surface.get_patches_slice() {
        if max_area.contains_point(&patch.area.point_center()) {
            total_in_area += 1;
        }
    }
    info!("witihin area {max_area} is {total_in_area} patches");

    draw_prep(surface, &select_batches);
    // if let Err(()) = debug_conflict_no_touching(surface, &select_batches) {
    //     error!("no touching");
    //     return;
    // } else {
    //     error!("good touching");
    // }

    for (i, batch) in select_batches.into_iter().enumerate() {
        trace!("batch {i}");
        let plan = get_possible_routes_for_batch(&surface, batch);
        debug_draw_complete_plan(surface, plan)?;
    }
    surface.save_pixel_to_oculante_zoomed();

    Ok(())
}
