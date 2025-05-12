use crate::navigator::mine_permutate::get_possible_routes_for_batch;
use crate::navigator::mine_selector::select_mines_and_sources;
use crate::navigator::planners::common::{debug_draw_complete_plan, draw_prep};
use crate::surfacev::err::VResult;
use crate::surfacev::vsurface::VSurface;
use itertools::Itertools;
use tracing::{info, trace};

pub fn start_debug_planner(surface: &mut VSurface) -> VResult<()> {
    let select_batches = select_mines_and_sources(&surface, 1)
        .into_success()
        .unwrap();

    draw_prep(surface, &select_batches);
    // if let Err(()) = debug_conflict_no_touching(surface, &select_batches) {
    //     error!("no touching");
    //     return;
    // } else {
    //     error!("good touching");
    // }

    info!("selected {} batches", select_batches.len());
    for (i, batch) in select_batches.into_iter().enumerate() {
        trace!("batch {i}");
        let plan = get_possible_routes_for_batch(&surface, batch);
        debug_draw_complete_plan(surface, plan)?;
    }
    surface.save_pixel_to_oculante();

    Ok(())
}
