use crate::always_true_test;
use crate::navigator::mine_executor::{ExecutorResult, FailingMeta, execute_route_batch};
use crate::navigator::mine_permutate::get_possible_routes_for_batch;
use crate::navigator::mine_selector::{MineSelectBatch, select_mines_and_sources};
use crate::navigator::planners::common::{PathingTunables, debug_failing, draw_prep};
use crate::state::machine::StepParams;
use crate::state::tuneables::Tunables;
use crate::surface::metric::Metrics;
use crate::surface::pixel::Pixel;
use crate::surfacev::vsurface::{VSurface, VSurfacePixelPatches, VSurfacePixelPatchesMut};
use std::path::Path;
use tracing::{error, info, trace, warn};

const RUZE_MAXIMUM_MINE_COUNT_PER_BATCH: usize = 5;

/// Planner v1 "Crimzon Ruze 💢"
///
/// Super parallel batch based planner
pub fn start_ruze_planner(
    tunables: &PathingTunables,
    surface: &mut VSurfacePixelPatchesMut,
    params: &StepParams,
) {
    let select_batches =
        select_mines_and_sources(tunables, surface, RUZE_MAXIMUM_MINE_COUNT_PER_BATCH)
            .into_success()
            .unwrap();

    let mut num_mines_metrics = Metrics::new("mine_batch_size");
    for batch in &select_batches {
        let total = batch.mines.len();
        num_mines_metrics.increment(format!("{total}"));
    }
    num_mines_metrics.log_final();

    draw_prep(surface, &select_batches);

    for (batch_index, batch) in select_batches.into_iter().enumerate() {
        // for (batch_index, batch) in [select_batches.into_iter().enumerate().last().unwrap()] {
        let found = process_batch(surface, batch, batch_index, &params.step_out_dir);
        if !found {
            error!("KILLING EARLY index {batch_index}");
            break;
        }

        // if 1 + 1 == 2 {
        if batch_index > 20 {
            break;
        }
    }
}

fn process_batch(
    surface: &mut VSurface,
    batch: MineSelectBatch,
    batch_index: usize,
    step_out_dir: &Path,
) -> bool {
    trace!("---");
    let num_mines = batch.mines.len();

    for mine in &batch.mines {
        mine.draw_area_buffered_to_no_touch(surface);
    }
    let complete_plan = get_possible_routes_for_batch(surface, batch);

    let num_per_batch_routes_min = complete_plan
        .sequences
        .iter()
        .map(|v| v.routes.len())
        .min()
        .unwrap();
    let num_per_batch_routes_max = complete_plan
        .sequences
        .iter()
        .map(|v| v.routes.len())
        .max()
        .unwrap();
    let num_routes_total: usize = complete_plan.sequences.iter().map(|v| v.routes.len()).sum();
    let num_batches = complete_plan.sequences.len();
    info!(
        "batch #{batch_index} with {num_mines} mines created {num_batches} combinations \
                with total routes {num_routes_total} \
                each in range {num_per_batch_routes_min} {num_per_batch_routes_max}"
    );
    // let planned_combinations = vec![planned_combinations.remove(0)];
    let res = execute_route_batch(surface, complete_plan.sequences, &[]); // todo: Shrink flag??
    match res {
        ExecutorResult::Success { paths, routes } => {
            info!("pushing {} new mine paths", paths.len());
            assert!(!paths.is_empty(), "Success but no paths!!!!");
            for route in routes {
                route.location.draw_area_buffered(surface);
            }
            complete_plan
                .base_sources
                .borrow_mut()
                .advance_by(paths.len())
                .unwrap();
            for path in paths {
                surface.add_mine_path(path);
            }
            true
        }
        ExecutorResult::Failure { meta, .. } => {
            if always_true_test() {
                debug_failing(surface, meta);
                return false;
            }

            let FailingMeta {
                found_paths,
                all_routes: failing_routes,
                astar_err,
            } = meta;

            error!("failed to pathfind");
            for path in found_paths {
                surface.add_mine_path_with_pixel(path, Pixel::Highlighter);
            }

            let (trigger_mine, rest) = failing_routes.split_first().unwrap();
            warn!(
                "trigger failing at {:?} with rest num {}",
                trigger_mine.location.area_buffered(),
                rest.len()
            );
            trigger_mine
                .location
                .draw_area_buffered_with(surface, Pixel::Highlighter);
            for entry in rest {
                warn!("failing at {:?}", entry.location.area_buffered());
                trigger_mine
                    .location
                    .draw_area_buffered_with(surface, Pixel::EdgeWall);
            }

            // // very busy dump
            // let mut pixels = Vec::new();
            // for dump_link in failing_dump {
            //     pixels.extend(dump_link.area());
            // }
            // surface.set_pixels(Pixel::Water, pixels).unwrap();
            // // then the targets
            // let mut pixels = Vec::new();
            // for locations in failing_routes {
            //     pixels.push(*locations.base_source.point());
            //     pixels.push(*locations.destination.point());
            // }
            // surface.set_pixels(Pixel::SteelChest, pixels).unwrap();

            // let mut all_pos = failing_all.iter().map(|v| v.next_straight_position()).collect_vec();
            // if debug_tree.is_empty() {
            //     warn!("gradient dump disabled");
            // } else {
            //     trace!("dumping {}", failing_all.len());
            //     let mut compressed: HashMap<VPoint, usize> = HashMap::new();
            //     for each in failing_all {
            //         let val = compressed.entry(each.pos_next()).or_default();
            //         *val += 1;
            //     }
            //     surface
            //         .save_pixel_img_colorized_grad_disk(step_out_dir, compressed)
            //         .unwrap();
            // }

            false
        }
    }
}
