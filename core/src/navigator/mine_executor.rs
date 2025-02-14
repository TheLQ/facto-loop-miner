use crate::navigator::mine_permutate::{PlannedBatch, PlannedRoute};
use crate::navigator::mori::{mori2_start, MoriResult};
use crate::surfacev::mine::MinePath;
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
use tracing::{debug, info};

pub const MINE_FRONT_RAIL_STEPS: usize = 6;

/// Given thousands of possible route combinations, execute in parallel and find the best
pub fn execute_route_batch(
    surface: &VSurface,
    planned_combinations: Vec<PlannedBatch>,
) -> MineRouteCombinationPathResult {
    let total_planned_combinations = planned_combinations.len();

    // At this point
    //  - Surface is modified from disk with no-touching-zones + other changes
    //  - Each thread needs to copy and modify its own Surface to work through a combination
    //
    // The mmap'd backed VArray can be re-mmap'd very quickly via clone
    // HOWEVER disk and memory must be the same / is_dirty=false / memory is unmodified
    // Caller will write our output result to the surface, then we repeat this safe/load
    let temp_executor_path = PathBuf::from("work/temp_executor");
    if let Err(err) = create_dir(&temp_executor_path) {
        debug!("recreating temp dir {}", temp_executor_path.display());
        remove_dir_all(&temp_executor_path).unwrap();
        create_dir(&temp_executor_path).unwrap();
    } else {
        debug!("created temp dir {}", temp_executor_path.display());
    }
    surface.save(&temp_executor_path).unwrap();
    let execution_surface = VSurface::load(&temp_executor_path).unwrap();
    // let execution_surface = surface;

    // debug: Get all the original patches
    // ???

    // reset counters
    TOTAL_COUNTER.store(0, Ordering::Relaxed);
    SUCCESS_COUNTER.store(0, Ordering::Relaxed);
    FAIL_COUNTER.store(0, Ordering::Relaxed);

    let routing_watch = BasicWatch::start();
    let default_threads = rayon::current_num_threads();
    const THREAD_OVERSUBSCRIBE_PERCENT: f32 = 1.5;
    let num_threads = (default_threads as f32 * THREAD_OVERSUBSCRIBE_PERCENT) as usize;
    info!(
        "default threads are {} upgraded to {}",
        default_threads, num_threads
    );
    let wrapping_pool = rayon::ThreadPoolBuilder::new()
        .thread_name(|i| format!("exe{i:02}"))
        .num_threads(num_threads)
        .build()
        .unwrap();

    let route_results: Vec<MineRouteCombinationPathResult> = wrapping_pool.install(|| {
        planned_combinations
            .into_par_iter()
            .map(|route_combination| {
                execute_route_combination(
                    &execution_surface,
                    route_combination,
                    total_planned_combinations,
                )
            })
            .collect()
    });

    debug!("Executed {total_planned_combinations} route combinations in {routing_watch}");

    let mut best_path: Option<MineRouteCombinationPathResult> = None;
    let mut lowest_cost = u32::MAX;
    let mut highest_cost = 0;
    let mut success_count = 0;
    let mut failure_found_paths_count: HashMap<usize, u32> = HashMap::new();
    for route_result in route_results {
        match &route_result {
            MineRouteCombinationPathResult::Success { paths } => {
                success_count += 1;

                let total_cost = paths.iter().map(|v| v.cost).sum();
                if total_cost < lowest_cost {
                    lowest_cost = total_cost;
                    best_path = Some(route_result);
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
                if best_path.is_none() {
                    best_path = Some(route_result);
                }
            }
        }
    }
    let failure_found_paths_count_debug = failure_found_paths_count
        .into_iter()
        .sorted_by_key(|(k, _v)| *k)
        .map(|(k, v)| format!("{}:{}", k, v))
        .join("|");
    info!(
        "Route batch of {total_planned_combinations} combinations had {} success, cost range {} to {}, failure {}, thread oversubscribe {}",
        success_count,
        highest_cost.to_formatted_string(&LOCALE),
        lowest_cost.to_formatted_string(&LOCALE),
        failure_found_paths_count_debug,
        THREAD_OVERSUBSCRIBE_PERCENT
    );

    best_path.unwrap()
}

static TOTAL_COUNTER: AtomicUsize = AtomicUsize::new(0);
static SUCCESS_COUNTER: AtomicUsize = AtomicUsize::new(0);
static FAIL_COUNTER: AtomicUsize = AtomicUsize::new(0);

fn execute_route_combination(
    surface: &VSurface,
    route_combination: PlannedBatch,
    total_planned_combinations: usize,
) -> MineRouteCombinationPathResult {
    let my_counter = TOTAL_COUNTER.fetch_add(1, Ordering::Relaxed);
    if my_counter % 100 == 0 {
        info!(
            "Processed {} of {} combinations, success {} fail {}",
            my_counter.to_formatted_string(&LOCALE),
            total_planned_combinations.to_formatted_string(&LOCALE),
            SUCCESS_COUNTER
                .load(Ordering::Relaxed)
                .to_formatted_string(&LOCALE),
            FAIL_COUNTER
                .load(Ordering::Relaxed)
                .to_formatted_string(&LOCALE),
        )
    }

    let watch = BasicWatch::start();
    let mut working_surface = (*surface).clone();
    info!("Cloned surface in {}", watch);

    let mut found_paths = Vec::new();
    for route in route_combination.routes {
        // let extended_entry_rails =
        //     match extend_rail_end(&working_surface, search_area, &mine_endpoint.entry_rail) {
        //         Some(v) => v,
        //         None => {
        //             // This was valid during first pass but now another Rail is on-top of it
        //             FAIL_COUNTER.fetch_add(1, Ordering::Relaxed);
        //             return MineRouteCombinationPathResult::Failure {
        //                 found_paths,
        //                 failing_mine: mine_endpoint.clone(),
        //             };
        //         }
        //     };

        let route_result = mori2_start(
            &working_surface,
            route.base_source,
            route.destination,
            &route.finding_limiter,
        );
        match route_result {
            MoriResult::Route { path, cost } => {
                // path.extend(extended_entry_rails);

                let path = MinePath {
                    links: path,
                    cost,
                    mine_base: route.location,
                };
                found_paths.push(path.clone());
                working_surface.add_mine_path(path).unwrap();
            }
            MoriResult::FailingDebug(debug_rail) => {
                FAIL_COUNTER.fetch_add(1, Ordering::Relaxed);
                return MineRouteCombinationPathResult::Failure {
                    found_paths,
                    failing_mine: route,
                };
            }
        }
    }
    SUCCESS_COUNTER.fetch_add(1, Ordering::Relaxed);
    MineRouteCombinationPathResult::Success { paths: found_paths }
}

//
// pub fn extend_rail_end(
//     surface: &VSurface,
//     search_area: &VArea,
//     mine_rail: &Rail,
// ) -> Option<Vec<Rail>> {
//     let mut rails: Vec<Rail> = Vec::new();
//     let mut last_rail = &mine_rail
//         .clone()
//         .into_buildable_simple(surface, search_area)?;
//     for i in 0..MINE_FRONT_RAIL_STEPS {
//         let rail = last_rail
//             .move_forward_step()
//             .into_buildable_simple(surface, search_area)?;
//         rails.push(rail);
//         last_rail = rails.last().unwrap();
//     }
//     Some(rails)
// }

pub enum MineRouteCombinationPathResult {
    Success {
        paths: Vec<MinePath>,
    },
    Failure {
        found_paths: Vec<MinePath>,
        failing_mine: PlannedRoute,
    },
}
