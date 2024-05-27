use crate::navigator::mori::{mori_start, Rail};
use crate::navigator::path_grouper::MineBase;
use crate::navigator::path_planner::{
    MineRouteCombination, MineRouteCombinationBatch, MineRouteEndpoints,
};
use crate::navigator::PathingResult;
use crate::surfacev::varea::VArea;
use crate::surfacev::vsurface::VSurface;
use crate::util::duration::BasicWatch;
use crate::LOCALE;
use num_format::ToFormattedString;
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;
use tracing::{debug, info};

///
pub fn execute_route_batch(
    surface: &VSurface,
    search_area: &VArea,
    route_batch: MineRouteCombinationBatch,
) -> Vec<MinePath> {
    let batch_size = route_batch.combinations.len();
    debug!("Executing {} route combination batch", batch_size);

    let routing_watch = BasicWatch::start();
    let route_results: Vec<MineRouteCombinationPathResult> = route_batch
        .combinations
        .into_par_iter()
        .map(|route_combination| execute_route_combination(surface, search_area, route_combination))
        .collect();
    debug!(
        "Executed {} route combinations in {}",
        batch_size, routing_watch
    );

    // We have many possible routes that have different costs. We want the lowest one
    let mut best_path = Vec::new();
    let mut best_cost = u32::MAX;
    let mut worst_cost = 0;
    let mut success_count = 0;
    for route_result in route_results {
        if let MineRouteCombinationPathResult::Success { paths } = route_result {
            success_count += 1;

            let total_cost = paths.iter().fold(0, |total, path| total + path.cost);
            if total_cost < best_cost {
                best_cost = total_cost;
                best_path = paths;
            }
            if total_cost > worst_cost {
                worst_cost = total_cost;
            }
        }
    }
    if !best_path.is_empty() {
        info!(
            "Route batch of {} combinations had {} success, cost range {} to {}",
            batch_size,
            success_count,
            worst_cost.to_formatted_string(&LOCALE),
            best_cost.to_formatted_string(&LOCALE)
        );
        best_path
    } else {
        panic!("Failed for {} combinations", batch_size)
    }
}

fn execute_route_combination(
    surface: &VSurface,
    search_area: &VArea,
    route_combination: MineRouteCombination,
) -> MineRouteCombinationPathResult {
    let mut found_paths = Vec::new();
    for route in route_combination.routes {
        let route_result = mori_start(
            surface,
            route.entry_rail.clone(),
            route.base_rail.clone(),
            search_area,
        );
        match route_result {
            PathingResult::Route { path, cost } => found_paths.push(MinePath {
                mine_base: route.mine,
                rail: path,
                cost,
            }),
            PathingResult::FailingDebug(debug_rail) => {
                return MineRouteCombinationPathResult::Failure {
                    found_paths,
                    failing_mine: route,
                }
            }
        }
    }
    MineRouteCombinationPathResult::Success { paths: found_paths }
}

pub struct MinePath {
    mine_base: MineBase,
    rail: Vec<Rail>,
    cost: u32,
}

pub enum MineRouteCombinationPathResult {
    Success {
        paths: Vec<MinePath>,
    },
    Failure {
        found_paths: Vec<MinePath>,
        failing_mine: MineRouteEndpoints,
    },
}
