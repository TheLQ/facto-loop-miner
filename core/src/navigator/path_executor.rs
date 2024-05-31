use crate::navigator::mori::{mori_start, write_rail, Rail};
use crate::navigator::path_grouper::MineBase;
use crate::navigator::path_planner::{
    MineRouteCombination, MineRouteCombinationBatch, MineRouteEndpoints,
};
use crate::navigator::PathingResult;
use crate::surfacev::varea::VArea;
use crate::surfacev::vsurface::VSurface;
use crate::util::duration::BasicWatch;
use crate::LOCALE;
use itertools::Itertools;
use num_format::ToFormattedString;
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;
use std::collections::HashMap;
use std::fs::{create_dir, remove_dir_all};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use tracing::{debug, error, info};

///
pub fn execute_route_batch(
    surface: &VSurface,
    route_batch: MineRouteCombinationBatch,
) -> Option<Vec<MinePath>> {
    let batch_size = route_batch.combinations.len();

    info!("Executing {} route combination batch", batch_size);

    // The backing entity_array files can be re-mmap'd very quickly via clone
    // HOWEVER disk and memory must be the same / is_dirty=false / memory is unmodified
    // Caller will write our output result to the surface, then we repeat this safe/load
    let path: PathBuf = PathBuf::from("work/temp_scan");
    // ignore
    if let Err(err) = create_dir(&path) {
        debug!("recreating temp dir {}", path.display());
        remove_dir_all(&path).unwrap();
        create_dir(&path).unwrap();
    } else {
        debug!("created temp dir {}", path.display());
    }
    surface.save(&path).unwrap();
    let execution_surface = VSurface::load(&path).unwrap();

    // debug: Get all the original patches

    let routing_watch = BasicWatch::start();
    let default_threads = rayon::current_num_threads();
    const THREAD_OVERSUBSCRIBE_PERCENT: f32 = 1.5;
    let num_threads = (default_threads as f32 * THREAD_OVERSUBSCRIBE_PERCENT) as usize;
    info!(
        "default threads are {} upgraded to {}",
        default_threads, num_threads
    );
    let wrapping_pool = rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build()
        .unwrap();

    let route_results: Vec<MineRouteCombinationPathResult> = wrapping_pool.install(|| {
        route_batch
            .combinations
            .into_par_iter()
            .map(|route_combination| {
                execute_route_combination(
                    &execution_surface,
                    &route_batch.planned_search_area,
                    route_combination,
                    batch_size,
                )
            })
            .collect()
    });

    debug!(
        "Executed {} route combinations in {}",
        batch_size, routing_watch
    );

    // We have many possible routes that have different costs. We want the lowest one
    let mut best_path = Vec::new();
    let mut lowest_cost = u32::MAX;
    let mut highest_cost = 0;
    let mut success_count = 0;

    let mut failure_found_paths_count: HashMap<usize, u32> = HashMap::new();
    for route_result in route_results {
        match route_result {
            MineRouteCombinationPathResult::Success { paths } => {
                success_count += 1;

                let total_cost = paths.iter().fold(0, |total, path| total + path.cost);
                if total_cost < lowest_cost {
                    lowest_cost = total_cost;
                    best_path = paths;
                }
                if total_cost > highest_cost {
                    highest_cost = total_cost;
                }
            }
            MineRouteCombinationPathResult::Failure {
                found_paths,
                failing_mine,
            } => {
                *failure_found_paths_count
                    .entry(found_paths.len())
                    .or_insert(0) += 1;
            }
        }
    }
    let failure_found_paths_count_debug = failure_found_paths_count
        .into_iter()
        .sorted_by_key(|(k, _v)| *k)
        .map(|(k, v)| format!("{}:{}", k, v))
        .join("|");
    info!(
        "Route batch of {} combinations had {} success, cost range {} to {}, failure {}, thread oversubscribe {}",
        batch_size,
        success_count,
        highest_cost.to_formatted_string(&LOCALE),
        lowest_cost.to_formatted_string(&LOCALE),
        failure_found_paths_count_debug,
        THREAD_OVERSUBSCRIBE_PERCENT
    );

    if !best_path.is_empty() {
        Some(best_path)
    } else {
        error!("Failed for {} combinations", batch_size);
        None
    }
}

fn execute_route_combination(
    surface: &VSurface,
    search_area: &VArea,
    route_combination: MineRouteCombination,
    batch_size: usize,
) -> MineRouteCombinationPathResult {
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    let my_counter = COUNTER.fetch_add(1, Ordering::Relaxed);
    if my_counter % 100 == 0 {
        info!(
            "Processed {} of {} combinations",
            my_counter.to_formatted_string(&LOCALE),
            batch_size.to_formatted_string(&LOCALE)
        )
    }

    // let watch = BasicWatch::start();
    let mut working_surface = (*surface).clone();
    // info!("Cloned surface in {}", watch);

    let mut found_paths = Vec::new();
    for route in route_combination.routes {
        let route_result = mori_start(
            &working_surface,
            route.base_rail.clone(),
            route.entry_rail.clone(),
            search_area,
        );
        match route_result {
            PathingResult::Route { path, cost } => {
                write_rail(&mut working_surface, &path).unwrap();
                found_paths.push(MinePath {
                    mine_base: route.mine,
                    rail: path,
                    cost,
                });
            }
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
    pub mine_base: MineBase,
    pub rail: Vec<Rail>,
    pub cost: u32,
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
