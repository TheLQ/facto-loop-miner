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
    pre_callback: impl Fn(&mut VSurface, &[ExecutionRoute], usize) + Send + Sync,
) -> ExecutorResult {
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
    let is_threaded = (sequences.len() > 1) ^ EXECUTE_THREADED;
    let route_results: Vec<ExecutorResult> = if is_threaded {
        let default_threads = 32; // todo: numa rayon::current_num_threads();
        const THREAD_OVERSUBSCRIBE_PERCENT: f32 = 1.0;
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
                    execute_route_combination(
                        execution_surface,
                        routes,
                        total_sequences,
                        &pre_callback,
                    )
                })
                .collect()
        })
    } else {
        sequences
            .into_iter()
            // .take(40)
            .map(|ExecutionSequence { routes }| {
                execute_route_combination(
                    &execution_surface,
                    routes,
                    total_sequences,
                    &pre_callback,
                )
            })
            .collect()
    };

    debug!("Executed {total_sequences} route combinations in {routing_watch}");

    struct CostMeta {
        lowest: u32,
        highest: u32,
        tested: u32,
    };
    impl CostMeta {
        fn new() -> Self {
            Self {
                lowest: u32::MAX,
                highest: u32::MIN,
                tested: 0,
            }
        }

        fn apply_and_is_lowest(&mut self, cost: u32) -> bool {
            self.tested += 1;
            self.highest = self.highest.max(cost);
            if cost < self.lowest {
                self.lowest = cost;
                true
            } else {
                false
            }
        }
    }
    let mut cost = CostMeta::new();

    let mut failure_attempts_per_len: HashMap<usize, u16> = HashMap::new();
    let mut success_count = 0;
    let mut failure_count = 0;
    let res: ExecutorResult = route_results.into_iter().fold(
        ExecutorResult::Failure(FailingMeta::default()),
        |best, cur_result| {
            let cur_paths = match &cur_result {
                ExecutorResult::Success { paths, .. } => {
                    success_count += 1;
                    paths
                }
                ExecutorResult::Failure(FailingMeta { found_paths, .. }) => {
                    let total = failure_attempts_per_len
                        .entry(found_paths.len())
                        .or_default();
                    *total += 1;
                    failure_count += 1;
                    found_paths
                }
            };
            let total_cost = cur_paths.iter().map(|v| v.cost).sum();

            match (&best, &cur_result) {
                (ExecutorResult::Success { .. }, ExecutorResult::Success { .. }) => {
                    if cost.apply_and_is_lowest(total_cost) {
                        cur_result
                    } else {
                        best
                    }
                }
                (ExecutorResult::Success { .. }, ExecutorResult::Failure { .. }) => {
                    // ignore failure after success
                    best
                }
                (ExecutorResult::Failure { .. }, ExecutorResult::Success { .. }) => {
                    // replace failure with success
                    cost = CostMeta::new();
                    cost.apply_and_is_lowest(total_cost);
                    cur_result
                }
                (
                    ExecutorResult::Failure(FailingMeta {
                        found_paths: best_paths,
                        ..
                    }),
                    ExecutorResult::Failure { .. },
                ) => {
                    if cur_paths.len() > best_paths.len() {
                        cost = CostMeta::new();
                        cost.apply_and_is_lowest(total_cost);
                        cur_result
                    } else {
                        if cost.apply_and_is_lowest(total_cost) {
                            cur_result
                        } else {
                            best
                        }
                    }
                }
            }
        },
    );

    let failure_attempts_debug = failure_attempts_per_len
        .into_iter()
        .sorted_by_key(|(k, _v)| *k)
        .map(|(k, v)| format!("{}:{}", k, v))
        .join("|");
    let deepest_depth = match &res {
        ExecutorResult::Failure { .. } => "FAIL".into(),
        ExecutorResult::Success { paths, .. } => {
            let mut deepest_depth = 0;
            for path in paths {
                deepest_depth = deepest_depth.max(path.links.len());
            }
            format!("{deepest_depth}")
        }
    };
    info!(
        "Route batch of {total_sequences} sequences had \
        {success_count} / {failure_count} success/failure, \
        cost range {} to {} (best {}), \
        attempts {failure_attempts_debug}, \
        depth {deepest_depth}, \
        res {}",
        cost.lowest.to_formatted_string(&LOCALE),
        cost.highest.to_formatted_string(&LOCALE),
        cost.tested,
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
    pre_callback: impl Fn(&mut VSurface, &[ExecutionRoute], usize),
) -> ExecutorResult {
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
    // info!("Cloned surface in {}", watch);

    let mut found_paths = Vec::new();
    for (i, route) in route_combination.iter().enumerate() {
        pre_callback(&mut working_surface, &route_combination, i);
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
                return ExecutorResult::Failure(FailingMeta {
                    all_routes: route_combination,
                    failing_all: debug_rail,
                    failing_dump: debug_all,
                    found_paths,
                });
            }
        }
    }

    SUCCESS_COUNTER.fetch_add(1, Ordering::Relaxed);
    ExecutorResult::Success {
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
pub enum ExecutorResult {
    Success {
        paths: Vec<MinePath>,
        routes: Vec<ExecutionRoute>,
    },
    Failure(FailingMeta),
}

#[derive(Default)]
pub struct FailingMeta {
    pub found_paths: Vec<MinePath>,
    pub all_routes: Vec<ExecutionRoute>,
    pub failing_dump: Vec<HopeLink>,
    pub failing_all: Vec<HopeLink>,
}
