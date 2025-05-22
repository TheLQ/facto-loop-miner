use crate::navigator::mori::{mori2_start, MoriResult};
use crate::surfacev::mine::{MineLocation, MinePath};
use crate::surfacev::vsurface::VSurface;
use facto_loop_miner_common::duration::BasicWatch;
use facto_loop_miner_common::LOCALE;
use facto_loop_miner_fac_engine::common::varea::VArea;
use facto_loop_miner_fac_engine::common::vpoint_direction::VSegment;
use facto_loop_miner_fac_engine::game_blocks::rail_hope_single::HopeLink;
use itertools::Itertools;
use num_format::ToFormattedString;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use strum::AsRefStr;
use tracing::{debug, info};

/// Given thousands of possible route combinations, execute in parallel and find the best
pub fn execute_route_batch(
    surface: &VSurface,
    sequences: Vec<ExecutionSequence>,
) -> MineRouteCombinationPathResult {
    let total_sequences = sequences.len();

    // TODO: This isn't actually saving time
    // // At this point
    // //  - Surface is modified from disk with no-touching-zones + other changes
    // //  - Each thread needs to copy and modify its own Surface to work through a combination
    // //
    // // The mmap'd backed VArray can be re-mmap'd very quickly via clone
    // // HOWEVER disk and memory must be the same / is_dirty=false / memory is unmodified
    // // Caller will write our output result to the surface, then we repeat this safe/load
    // let temp_executor_path = PathBuf::from("work/temp_executor");
    // if let Err(err) = create_dir(&temp_executor_path) {
    //     debug!("recreating temp dir {}", temp_executor_path.display());
    //     remove_dir_all(&temp_executor_path).unwrap();
    //     create_dir(&temp_executor_path).unwrap();
    // } else {
    //     debug!("created temp dir {}", temp_executor_path.display());
    // }
    // surface.save(&temp_executor_path).unwrap();
    // let execution_surface = &VSurface::load(&temp_executor_path).unwrap();
    let execution_surface = surface;

    // debug: Get all the original patches
    // ???

    // reset counters
    TOTAL_COUNTER.store(0, Ordering::Relaxed);
    SUCCESS_COUNTER.store(0, Ordering::Relaxed);
    FAIL_COUNTER.store(0, Ordering::Relaxed);

    let routing_watch = BasicWatch::start();

    const EXECUTE_THREADED: bool = true;
    let route_results: Vec<MineRouteCombinationPathResult> = if EXECUTE_THREADED {
        let default_threads = 32; // todo: numa rayon::current_num_threads();
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
        wrapping_pool.install(|| {
            sequences
                .into_par_iter()
                .map(|ExecutionSequence { routes }| {
                    execute_route_combination(execution_surface, routes, total_sequences)
                })
                .collect()
        })
    } else {
        sequences
            .into_iter()
            .map(|ExecutionSequence { routes }| {
                execute_route_combination(&execution_surface, routes, total_sequences)
            })
            .collect()
    };

    debug!("Executed {total_sequences} route combinations in {routing_watch}");

    let mut best_path: Option<MineRouteCombinationPathResult> = None;
    let mut lowest_cost = u32::MAX;
    let mut highest_cost = u32::MIN;
    let mut success_count = 0;
    let failure_found_paths_count: HashMap<usize, u32> = HashMap::new();
    for route_result in route_results {
        let paths = match &route_result {
            MineRouteCombinationPathResult::Success { paths, .. } => paths,
            MineRouteCombinationPathResult::Failure {
                meta: FailingMeta { found_paths, .. },
            } => found_paths,
        };
        let total_cost = paths.iter().map(|v| v.cost).sum();

        match &route_result {
            MineRouteCombinationPathResult::Success { .. } => {
                success_count += 1;

                match best_path {
                    Some(MineRouteCombinationPathResult::Failure { .. }) | None => {
                        best_path = Some(route_result);
                        lowest_cost = total_cost;
                        highest_cost = total_cost;
                    }
                    Some(MineRouteCombinationPathResult::Success { .. }) => {
                        if total_cost < lowest_cost {
                            lowest_cost = total_cost;
                            best_path = Some(route_result);
                        }
                        if total_cost > highest_cost {
                            highest_cost = total_cost;
                        }
                    }
                }
            }
            MineRouteCombinationPathResult::Failure { meta } => {
                let found_len = meta.found_paths.len();
                match best_path {
                    None => {
                        best_path = Some(route_result);
                        lowest_cost = total_cost;
                        highest_cost = total_cost;
                    }
                    Some(MineRouteCombinationPathResult::Failure { meta: prev_meta })
                        if found_len > prev_meta.found_paths.len() =>
                    {
                        best_path = Some(route_result);
                        lowest_cost = total_cost;
                        highest_cost = total_cost;
                    }
                    Some(MineRouteCombinationPathResult::Failure { .. }) => {
                        if total_cost < lowest_cost {
                            lowest_cost = total_cost;
                            best_path = Some(route_result);
                        }
                        if total_cost > highest_cost {
                            highest_cost = total_cost;
                        }
                    }
                    Some(MineRouteCombinationPathResult::Success { .. }) => {}
                }
            }
        }
    }
    let failure_found_paths_count_debug = failure_found_paths_count
        .into_iter()
        .sorted_by_key(|(k, _v)| *k)
        .map(|(k, v)| format!("{}:{}", k, v))
        .join("|");
    let res = best_path.unwrap();
    info!(
        "Route batch of {total_sequences} combinations had {} success, cost range {} to {}, failure {}, res {}",
        success_count,
        highest_cost.to_formatted_string(&LOCALE),
        lowest_cost.to_formatted_string(&LOCALE),
        failure_found_paths_count_debug,
        res.as_ref(),
    );

    res
}

static TOTAL_COUNTER: AtomicUsize = AtomicUsize::new(0);
static SUCCESS_COUNTER: AtomicUsize = AtomicUsize::new(0);
static FAIL_COUNTER: AtomicUsize = AtomicUsize::new(0);

fn execute_route_combination(
    surface: &VSurface,
    route_combination: Vec<ExecutionRoute>,
    total_sequences: usize,
) -> MineRouteCombinationPathResult {
    let my_counter = TOTAL_COUNTER.fetch_add(1, Ordering::Relaxed);
    if my_counter % 100 == 0 {
        info!(
            "Processed {} of {} sequences, success {} fail {}",
            my_counter.to_formatted_string(&LOCALE),
            total_sequences.to_formatted_string(&LOCALE),
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
    for (i, route) in route_combination.iter().enumerate() {
        let route_result = mori2_start(
            &working_surface,
            route.segment.clone(),
            &route.finding_limiter,
        );
        match route_result {
            MoriResult::Route { path, cost } => {
                // path.extend(extended_entry_rails);

                let path = MinePath {
                    links: path,
                    cost,
                    mine_base: route.location.clone(),
                    segment: route.segment.clone(),
                };
                found_paths.push(path.clone());
                working_surface.add_mine_path(path).unwrap();
            }
            MoriResult::FailingDebug(debug_rail, debug_all) => {
                FAIL_COUNTER.fetch_add(1, Ordering::Relaxed);
                return MineRouteCombinationPathResult::Failure {
                    meta: FailingMeta {
                        failing_routes: route_combination,
                        failing_all: debug_rail,
                        failing_dump: debug_all,
                        found_paths,
                    },
                };
            }
        }
    }

    SUCCESS_COUNTER.fetch_add(1, Ordering::Relaxed);
    MineRouteCombinationPathResult::Success {
        paths: found_paths,
        routes: route_combination,
    }
}

pub struct ExecutionRoute {
    pub location: MineLocation,
    pub segment: VSegment,
    pub finding_limiter: VArea,
}

pub struct ExecutionSequence {
    pub routes: Vec<ExecutionRoute>,
}

#[derive(AsRefStr)]
pub enum MineRouteCombinationPathResult {
    Success {
        paths: Vec<MinePath>,
        routes: Vec<ExecutionRoute>,
    },
    Failure {
        meta: FailingMeta,
    },
}

#[derive(Default)]
pub struct FailingMeta {
    pub found_paths: Vec<MinePath>,
    pub failing_routes: Vec<ExecutionRoute>,
    pub failing_dump: Vec<HopeLink>,
    pub failing_all: Vec<HopeLink>,
}
