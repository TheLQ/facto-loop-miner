use crate::navigator::mine_permutate::get_possible_routes_for_batch;
use crate::navigator::mine_selector::select_mines_and_sources;
use crate::navigator::planners::common::{debug_draw_complete_plans, draw_no_touching_zone};
use crate::surfacev::vsurface::VSurface;
use itertools::Itertools;

pub fn start_debug_planner(surface: &mut VSurface) {
    let select_batches = select_mines_and_sources(&surface, 1)
        .into_success()
        .unwrap();

    draw_no_touching_zone(surface, &select_batches);

    let plans = select_batches
        .into_iter()
        .map(|batch| get_possible_routes_for_batch(&surface, batch))
        .collect_vec();
    debug_draw_complete_plans(surface, plans);
}
