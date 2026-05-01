use crate::navigator::planners::PathingTunables;
use crate::surfacev::vsurface::VSurfacePatchMut;

// pub fn start_debug_planner(_tunables: &PathingTunables, _surface_mut: &mut VSurfacePatchMut) {
//     panic!("todo")
//     // let select_batches = get_batches(tunables, surface_mut.patches());
//     // paint_result(&mut surface_mut.pixels_mut(), select_batches);
//     // if let Err(()) = debug_conflict_no_touching(surface, &select_batches) {
//     //     error!("no touching");
//     //     return;
//     // } else {
//     //     error!("good touching");
//     // }
// }

// fn get_batches(tunables: &PathingTunables, surface: VSurfacePatch) -> Vec<MineSelectBatch> {
//     let select_batches = select_mines_and_sources(tunables, surface, 5)
//         .into_success()
//         .unwrap();
//     let mines: usize = select_batches
//         .iter()
//         .flat_map(|v| &v.mines)
//         .map(VSurfacePatch::mine_patches_len)
//         .sum();
//     info!(
//         "selected {mines} total patches in {} batches",
//         select_batches.len()
//     );
//
//     let max_area = VArea::from_arbitrary_points(
//         select_batches
//             .iter()
//             .flat_map(|v| &v.mines)
//             .flat_map(|v| v.area_min().get_corner_points()),
//     );
//     let mut total_in_area = 0;
//     for patch in surface.get_patches() {
//         if max_area.contains_point(&patch.area.point_center()) {
//             total_in_area += 1;
//         }
//     }
//     info!("witihin area {max_area} is {total_in_area} patches");
//     select_batches
// }

// fn paint_result(surface_mut: &mut VSurfacePixelMut, select_batches: Vec<MineSelectBatch>) {
//     draw_prep(surface_mut, &select_batches);
//     for (i, batch) in select_batches.into_iter().enumerate() {
//         trace!("batch {i}");
//         let plan = get_possible_routes_for_batch(surface_mut.pixels(), batch);
//         debug_draw_complete_plan(surface_mut, plan);
//     }
//     surface_mut
//         .pixels()
//         .paint_pixel_colored_zoomed()
//         .save_to_oculante();
// }
