use crate::navigator::mori::{MoriResult, mori2_start};
use crate::state::tuneables::MoriTunables;
use crate::surfacev::mine::{MineLocation, MinePath};
use crate::surfacev::vsurface::{
    VSurfacePixel, VSurfacePixelAsVs, VSurfacePixelAsVsMut, VSurfacePixelMut, VSurfaceRail,
    VSurfaceRailAsVsMut,
};
use facto_loop_miner_common::duration::BasicWatch;
use facto_loop_miner_common::{EXECUTOR_TAG, LOCALE};
use facto_loop_miner_fac_engine::common::varea::VArea;
use facto_loop_miner_fac_engine::common::vpoint_direction::VSegment;
use facto_loop_miner_fac_engine::game_blocks::rail_hope_soda::HopeSodaLink;
use itertools::Itertools;
use num_format::ToFormattedString;
use pathfinding::prelude::AStarErr;
use rayon::ThreadPool;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use std::collections::HashMap;
use std::sync::LazyLock;
use std::sync::atomic::{AtomicUsize, Ordering};
use strum::AsRefStr;
use tracing::{Level, info, span, trace};

pub fn execute_route_batch_clone_prep(
    tunables: &MoriTunables,
    surface: &mut VSurfacePixelMut,
    sequences: Vec<ExecutionSequence>,
    flags: &[ExecuteFlags],
) -> ExecutorResult {
    // At this point
    //  - Surface is modified from disk with no-touching-zones + other changes
    //  - Each thread needs to copy and modify its own Surface to work through a combination
    //
    // The mmap'd backed VArray can be re-mmap'd very quickly via clone
    // HOWEVER disk and memory must be the same / is_dirty=false / memory is unmodified
    // Caller will write our output result to the surface, then we repeat this safe/load
    surface.load_clone_prep().unwrap();

    execute_route_batch(tunables, surface.pixels(), sequences, flags)
}

/// Given thousands of possible route combinations, execute in parallel and find the best
pub fn execute_route_batch(
    tunables: &MoriTunables,
    execution_surface: VSurfacePixel,
    sequences: Vec<ExecutionSequence>,
    flags: &[ExecuteFlags],
) -> ExecutorResult {
    let total_sequences = sequences.len();
    let unique_mines = {
        let mut seen: Vec<&MineLocation> = Vec::new();
        for sequence in &sequences {
            for route in sequence.routes() {
                if !seen.contains(&&route.location) {
                    seen.push(&route.location);
                }
            }
        }
        seen.len()
    };

    // dedupe is bad
    {
        let seq_segments: Vec<Vec<VSegment>> = sequences
            .iter()
            .map(|v| v.routes().iter().map(|v| v.segment.clone()).collect())
            .collect();
        let mut seq_segments_clean = seq_segments.clone();
        seq_segments_clean.dedup();
        seq_segments_clean.sort();
        assert_eq!(
            seq_segments_clean.len(),
            total_sequences,
            "dedupe detected {}",
            seq_segments
                .iter()
                .map(|v| v.iter().map(|v| v.to_string()).join(","))
                .join("\n")
        );
        // trace!(
        //     "segments\n{}",
        //     seq_segments
        //         .iter()
        //         .map(|v| v.iter().map(|v| v.to_string()).join(","))
        //         .join("\n")
        // );
    };

    // reset counters
    TOTAL_COUNTER.store(0, Ordering::Relaxed);
    SUCCESS_COUNTER.store(0, Ordering::Relaxed);
    FAIL_COUNTER.store(0, Ordering::Relaxed);

    let execute_watch = BasicWatch::start();

    static WRAPPING_POOL: LazyLock<ThreadPool> = LazyLock::new(|| {
        let default_threads = 32; // todo: numa rayon::current_num_threads();
        const THREAD_OVERSUBSCRIBE_PERCENT: f32 = 1.0;
        let num_threads = (default_threads as f32 * THREAD_OVERSUBSCRIBE_PERCENT) as usize;
        info!(
            "default threads are {} upgraded to {}",
            default_threads, num_threads
        );
        rayon::ThreadPoolBuilder::new()
            .thread_name(|i| format!("exe{i:02}"))
            .num_threads(num_threads)
            .build()
            .unwrap()
    });

    const EXECUTE_THREADED: bool = true;
    let is_threaded = (sequences.len() > 1) && EXECUTE_THREADED;
    let route_results: Vec<ExecutorResult> = if is_threaded {
        WRAPPING_POOL.install(|| {
            sequences
                .into_par_iter()
                .map(|sequence| {
                    execute_route_combination(
                        tunables,
                        execution_surface,
                        sequence,
                        total_sequences,
                        flags,
                    )
                })
                .collect()
        })
    } else {
        sequences
            .into_iter()
            // .take(40)
            .map(|sequence| {
                execute_route_combination(
                    tunables,
                    execution_surface,
                    sequence,
                    total_sequences,
                    flags,
                )
            })
            .collect()
    };

    let execute_watch = execute_watch.to_string();
    // debug!("Executed {total_sequences} route combinations in {routing_watch}");

    struct CostMeta {
        lowest: u32,
        highest: u32,
        tested: u32,
    }
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
    let mut failure_seen_mines = HashMap::new();
    let mut success_count = 0;
    let mut failure_count = 0;
    let res: ExecutorResult = route_results.into_iter().fold(
        ExecutorResult::Failure {
            meta: FailingMeta::default(),
            seen_mines: HashMap::new(),
        },
        |best, cur_result| {
            let cur_paths = match &cur_result {
                ExecutorResult::Success { paths, .. } => {
                    success_count += 1;
                    paths
                }
                ExecutorResult::Failure {
                    meta: FailingMeta { found_paths, .. },
                    ..
                } => {
                    let total = failure_attempts_per_len
                        .entry(found_paths.len())
                        .or_default();
                    *total += 1;
                    failure_count += 1;
                    found_paths
                }
            };
            let total_cost = cur_paths.iter().map(|v| v.cost).sum();

            if let ExecutorResult::Failure { meta, .. } = &cur_result {
                for path in &meta.found_paths {
                    let next = failure_seen_mines.entry(path.location.clone()).or_default();
                    *next += 1;
                }
            }

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
                    ExecutorResult::Failure {
                        meta: best_meta, ..
                    },
                    ExecutorResult::Failure { meta: cur_meta, .. },
                ) => {
                    if best_meta.failing_sequence < cur_meta.failing_sequence {
                        cost = CostMeta::new();
                        cost.apply_and_is_lowest(total_cost);
                        cur_result
                    } else if cost.apply_and_is_lowest(total_cost) {
                        cur_result
                    } else {
                        best
                    }
                }
            }
        },
    );
    // merge extra tracked state
    let res = match res {
        ExecutorResult::Failure { meta, seen_mines } => {
            assert_eq!(seen_mines.len(), 0);
            ExecutorResult::Failure {
                meta,
                seen_mines: failure_seen_mines,
            }
        }
        r => r,
    };

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
    let mode = if is_threaded { "P" } else { "S" };
    info!(
        "Batch {mode} of {total_sequences} sequences had \
        {success_count} / {failure_count} success/failure, \
        cost range {} .. {} (best {}), \
        attempts {failure_attempts_debug}, \
        mines {unique_mines}, \
        depth {deepest_depth}, \
        res {}, \
        exec {execute_watch}",
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
    tuneables: &MoriTunables,
    surface: VSurfacePixel,
    sequence: ExecutionSequence,
    total_sequences: usize,
    flags: &[ExecuteFlags],
) -> ExecutorResult {
    let executor_mark = span!(Level::INFO, EXECUTOR_TAG);
    let _mark = executor_mark.enter();
    let my_counter = TOTAL_COUNTER.fetch_add(1, Ordering::Relaxed);
    if my_counter.is_multiple_of(100) {
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

    // let watch = BasicWatch::start();
    let mut surface_copy = VSurfaceRail::surface_copy(surface);
    let surface = &mut surface_copy.rails_mut();
    // info!("Cloned surface in {}", watch);

    for (i, route) in sequence.routes().iter().enumerate() {
        if flags.contains(&ExecuteFlags::ShrinkBases) {
            route
                .location
                .draw_area_buffered_to_no_touch(&mut surface.pixels_mut());
            if i != 0 {
                sequence.routes()[i - 1]
                    .location
                    .draw_area_buffered(&mut surface.pixels_mut())
            }
        }

        trace!(
            "for mine {} endpoints {}",
            route.location.area_min().point_center(),
            route
                .location
                .destinations()
                .map(|v| v.to_string())
                .join(",")
        );
        let route_result = mori2_start(
            tuneables,
            surface.pixels(),
            route.segment.clone(),
            &route.finding_limiter,
        );
        match route_result {
            MoriResult::Route { path, sodas, cost } => {
                // path.extend(extended_entry_rails);

                let path = MinePath {
                    links: path,
                    sodas,
                    cost,
                    location: route.location.clone(),
                    segment: route.segment.clone(),
                };
                surface.add_mine_path(path);
            }
            MoriResult::FailingDebug { err } => {
                FAIL_COUNTER.fetch_add(1, Ordering::Relaxed);
                return ExecutorResult::Failure {
                    meta: FailingMeta {
                        sequence,
                        failing_sequence: FailingSequence(i),
                        astar_err: err,
                        found_paths: surface_copy.into_rails(),
                    },
                    seen_mines: HashMap::new(),
                };
            }
        }
    }

    SUCCESS_COUNTER.fetch_add(1, Ordering::Relaxed);
    ExecutorResult::Success {
        paths: surface_copy.into_rails(),
        sequence,
    }
}

pub struct ExecutionRoute {
    pub location: MineLocation,
    pub segment: VSegment,
    pub finding_limiter: VArea,
}

/// A single attempt of routes
pub struct ExecutionSequence(Vec<ExecutionRoute>);

impl ExecutionSequence {
    pub fn new(routes: Vec<ExecutionRoute>) -> Self {
        Self(routes)
    }

    pub fn routes(&self) -> &[ExecutionRoute] {
        &self.0
    }

    pub fn split_routes_from(&self, failing_sequence: FailingSequence) -> ExecutionSequenceParts {
        let (pass, fail) = self.0.split_at(failing_sequence.0);
        ExecutionSequenceParts { pass, fail }
    }
}

pub struct ExecutionSequenceParts<'r> {
    pub pass: &'r [ExecutionRoute],
    pub fail: &'r [ExecutionRoute],
}

#[derive(AsRefStr)]
pub enum ExecutorResult {
    Success {
        paths: Vec<MinePath>,
        sequence: ExecutionSequence,
    },
    Failure {
        meta: FailingMeta,
        seen_mines: HashMap<MineLocation, usize>,
    },
}

impl ExecutorResult {
    fn get_sequence(&self) -> &ExecutionSequence {
        match self {
            ExecutorResult::Success { sequence, .. } => sequence,
            ExecutorResult::Failure {
                meta: FailingMeta { sequence, .. },
                ..
            } => sequence,
        }
    }
}

// #[derive(Default)]
pub struct FailingMeta {
    pub found_paths: Vec<MinePath>,
    pub sequence: ExecutionSequence,
    pub failing_sequence: FailingSequence,
    pub astar_err: AStarErr<HopeSodaLink, u32>,
}

impl Default for FailingMeta {
    fn default() -> Self {
        Self {
            astar_err: AStarErr {
                seen: Vec::new(),
                parents: Default::default(),
            },
            found_paths: Vec::new(),
            failing_sequence: FailingSequence(usize::MAX),
            sequence: ExecutionSequence(Vec::new()),
        }
    }
}

#[repr(transparent)]
#[derive(PartialEq, PartialOrd)]
pub struct FailingSequence(usize);

#[derive(PartialEq)]
pub enum ExecuteFlags {
    ShrinkBases,
}
