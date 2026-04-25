use crate::navigator::BaseSourceEighth;
use crate::navigator::mori::{MoriResult, mori2_start};
use crate::state::tuneables::MoriTunables;
use crate::surfacev::mine::{MineDestination, MineLocation, MinePath};
use crate::surfacev::vsurface::{
    VSurfacePixel, VSurfacePixelAsVs, VSurfacePixelAsVsMut, VSurfacePixelMut, VSurfaceRail,
    VSurfaceRailAsVsMut,
};
use facto_loop_miner_common::duration::BasicWatch;
use facto_loop_miner_common::{EXECUTOR_TAG, LOCALE};
use facto_loop_miner_fac_engine::common::varea::VArea;
use facto_loop_miner_fac_engine::common::vpoint::VPoint;
use facto_loop_miner_fac_engine::common::vpoint_direction::VSegment;
use facto_loop_miner_fac_engine::game_blocks::rail_hope_soda::HopeSodaLink;
use itertools::Itertools;
use num_format::ToFormattedString;
use pathfinding::prelude::AStarErr;
use rayon::ThreadPool;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use simd_json::prelude::ArrayTrait;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::sync::LazyLock;
use std::sync::atomic::{AtomicUsize, Ordering};
use strum::AsRefStr;
use tracing::{Level, info, span, trace};

pub fn execute_route_batch_clone_prep<'plan_mine>(
    tunables: &MoriTunables,
    surface: &mut VSurfacePixelMut,
    sequences: Vec<ExecutionSequence<'plan_mine>>,
    base_source: &BaseSourceEighth,
    flags: &[ExecuteFlags],
) -> ExecutorResult<'plan_mine> {
    // At this point
    //  - Surface is modified from disk with no-touching-zones + other changes
    //  - Each thread needs to copy and modify its own Surface to work through a combination
    //
    // The mmap'd backed VArray can be re-mmap'd very quickly via clone
    // HOWEVER disk and memory must be the same / is_dirty=false / memory is unmodified
    // Caller will write our output result to the surface, then we repeat this safe/load
    surface.load_clone_prep().unwrap();

    execute_route_batch(tunables, surface.pixels(), sequences, base_source, flags)
}

/// Given thousands of possible route combinations, execute in parallel and find the best
pub fn execute_route_batch<'plan_mine>(
    tunables: &MoriTunables,
    execution_surface: VSurfacePixel,
    sequences: Vec<ExecutionSequence<'plan_mine>>,
    base_source: &BaseSourceEighth,
    flags: &[ExecuteFlags],
) -> ExecutorResult<'plan_mine> {
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
        let seq_segments: Vec<Vec<(&MineDestination, &MineLocation)>> = sequences
            .iter()
            .map(|v| {
                v.routes()
                    .iter()
                    .map(|v| (v.destination, v.location))
                    .collect()
            })
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
                .map(|v| v
                    .iter()
                    .map(|(dest, loc)| format!("{dest:?} - {loc:?}"))
                    .join(","))
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
    let route_results: Vec<ExecutorThreadResult> = if is_threaded {
        WRAPPING_POOL.install(|| {
            sequences
                .into_par_iter()
                .map(|sequence| {
                    execute_sequence(
                        tunables,
                        execution_surface,
                        sequence,
                        base_source,
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
                execute_sequence(
                    tunables,
                    execution_surface,
                    sequence,
                    base_source,
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
    let mut success_count = 0;
    let mut failure_count = 0;
    let res: ExecutorResult = route_results
        .into_iter()
        .fold::<Option<ExecutorResult>, _>(None, |best, cur_result| {
            let mut best = match best {
                Some(v) => v,
                None => return Some(cur_result.into_main_result()),
            };

            let cur_paths = match &cur_result {
                ExecutorThreadResult::Success { paths, .. } => {
                    success_count += 1;
                    paths
                }
                ExecutorThreadResult::Failure {
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

            match (&mut best, cur_result) {
                (
                    ExecutorResult::Success { .. },
                    cur_result @ ExecutorThreadResult::Success { .. },
                ) => {
                    if cost.apply_and_is_lowest(total_cost) {
                        Some(cur_result.into_main_result())
                    } else {
                        Some(best)
                    }
                }
                (ExecutorResult::Success { .. }, ExecutorThreadResult::Failure { .. }) => {
                    // ignore failure after success
                    Some(best)
                }
                (
                    ExecutorResult::Failure { .. },
                    cur_result @ ExecutorThreadResult::Success { .. },
                ) => {
                    // replace failure with success
                    cost = CostMeta::new();
                    cost.apply_and_is_lowest(total_cost);
                    Some(cur_result.into_main_result())
                }
                (
                    ExecutorResult::Failure { stats },
                    ExecutorThreadResult::Failure { meta: cur_meta, .. },
                ) => {
                    if stats.is_more_failing_sequence(&cur_meta) {
                        cost = CostMeta::new();
                    }
                    stats.absorb_meta(cur_meta);
                    if cost.apply_and_is_lowest(total_cost) {
                        stats.last_best_meta();
                    }
                    // return with updated stats
                    Some(best)
                }
            }
        })
        .unwrap();

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

fn execute_sequence<'plan_mine>(
    tuneables: &MoriTunables,
    surface: VSurfacePixel,
    sequence: ExecutionSequence<'plan_mine>,
    base_source: &BaseSourceEighth,
    total_sequences: usize,
    flags: &[ExecuteFlags],
) -> ExecutorThreadResult<'plan_mine> {
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
    let mut surface_copy = VSurfaceRail::surface_copy_no_rails(surface);
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

        // trace!(
        //     "for mine {} endpoints {}",
        //     route.location.area_min().point_center(),
        //     route
        //         .location
        //         .destinations_for_level(&route.intra_level)
        //         .map(|v| v.to_string())
        //         .join(",")
        // );
        let source = base_source.peek_after(i);
        let segment = route.segment_for_source(&source);
        let route_result = mori2_start(
            tuneables,
            surface.pixels(),
            segment.clone(),
            &route.finding_limiter,
        );
        match route_result {
            MoriResult::Route { path, sodas, cost } => {
                // path.extend(extended_entry_rails);

                let path = MinePath {
                    links: path,
                    sodas,
                    cost,
                    location: route.location.actually_clone(),
                    segment,
                };
                surface.add_mine_path(path);
            }
            MoriResult::FailingDebug { cause } => {
                FAIL_COUNTER.fetch_add(1, Ordering::Relaxed);
                return ExecutorThreadResult::Failure {
                    meta: FailingMeta {
                        sequence,
                        failing_sequence: FailingSequence::new(i),
                        cause,
                        found_paths: surface_copy.into_rails(),
                    },
                };
            }
        }
    }

    SUCCESS_COUNTER.fetch_add(1, Ordering::Relaxed);
    ExecutorThreadResult::Success {
        paths: surface_copy.into_rails(),
        sequence,
    }
}

pub struct ExecutionRoute<'plan_mine> {
    pub location: &'plan_mine MineLocation,
    pub destination: &'plan_mine MineDestination,
    pub finding_limiter: VArea,
}

impl ExecutionRoute<'_> {
    pub fn segment_for_source(&self, source: &BaseSourceEntry) -> VSegment {
        source.segment_for_mine(self.destination)
    }
}

/// A single attempt of routes
pub struct ExecutionSequence<'plan_mine>(Vec<ExecutionRoute<'plan_mine>>);

impl<'plan_mine> ExecutionSequence<'plan_mine> {
    pub fn new(routes: Vec<ExecutionRoute<'plan_mine>>) -> Self {
        Self(routes)
    }

    pub fn routes(&self) -> &[ExecutionRoute<'plan_mine>] {
        &self.0
    }

    pub fn split_routes_from(
        &self,
        failing_sequence: FailingSequence,
    ) -> ExecutionSequenceParts<'_, 'plan_mine> {
        let (pass, fail) = failing_sequence.split_at(self.0.as_slice());
        ExecutionSequenceParts { pass, fail }
    }
}

pub struct ExecutionSequenceParts<'r, 'plan_mine> {
    pub pass: &'r [ExecutionRoute<'plan_mine>],
    pub fail: &'r [ExecutionRoute<'plan_mine>],
}

#[derive(AsRefStr)]
pub enum ExecutorResult<'plan_mine> {
    Success {
        paths: Vec<MinePath>,
        sequence: ExecutionSequence<'plan_mine>,
    },
    Failure {
        stats: FailingStats<'plan_mine>,
    },
}

pub enum ExecutorThreadResult<'plan_mine> {
    Success {
        paths: Vec<MinePath>,
        sequence: ExecutionSequence<'plan_mine>,
    },
    Failure {
        meta: FailingMeta<'plan_mine>,
    },
}

impl<'plan_mine> ExecutorThreadResult<'plan_mine> {
    fn into_main_result(self) -> ExecutorResult<'plan_mine> {
        match self {
            ExecutorThreadResult::Success { paths, sequence } => {
                ExecutorResult::Success { paths, sequence }
            }
            ExecutorThreadResult::Failure { meta } => ExecutorResult::Failure {
                stats: FailingStats::new(meta),
            },
        }
    }
}

// #[derive(Default)]
pub struct FailingMeta<'plan_mine> {
    pub found_paths: Vec<MinePath>,
    pub sequence: ExecutionSequence<'plan_mine>,
    pub failing_sequence: FailingSequence,
    pub cause: FailingCause,
}

pub enum FailingCause {
    AStar(AStarErr<HopeSodaLink, u32>),
    Wasted(Vec<VPoint>),
}

// impl Default for FailingMeta {
//     fn default() -> Self {
//         Self {
//             astar_err: AStarErr {
//                 seen: Vec::new(),
//                 parents: Default::default(),
//             },
//             found_paths: Vec::new(),
//             failing_sequence: FailingSequence(usize::MAX),
//             sequence: ExecutionSequence(Vec::new()),
//         }
//     }
// }

pub struct FailingStats<'plan_mine> {
    pub last_meta: Option<FailingMeta<'plan_mine>>,
    pub best_meta: Option<FailingMeta<'plan_mine>>,
    /// MineLocation is owned by best_meta and last_meta chain. Can't make a ref
    /// avoid further lifetype noise with MineLocation.to_string()
    pub seen_mines: SeenMines<'plan_mine>,
    pub seen_destinations: HashMap<VPoint, usize>,
    pub wasted_per_len: HashMap<usize, usize>,
    pub failures_per_len: HashMap<usize, usize>,
    pub wasteds: HashMap<Vec<VPoint>, usize>,
}

impl<'plan_mine> FailingStats<'plan_mine> {
    fn new(best_meta: FailingMeta<'plan_mine>) -> Self {
        let mut new = Self {
            last_meta: None,
            best_meta: None,
            seen_mines: SeenMines(HashMap::new()),
            seen_destinations: HashMap::new(),
            wasted_per_len: HashMap::new(),
            failures_per_len: HashMap::new(),
            wasteds: HashMap::new(),
        };
        new.absorb_meta(best_meta);
        new.last_best_meta();
        new
    }

    fn absorb_meta(&mut self, meta: FailingMeta<'plan_mine>) {
        let failures_at_len = self
            .failures_per_len
            .entry(meta.found_paths.len())
            .or_default();
        *failures_at_len += 1;

        match &meta.cause {
            FailingCause::Wasted(wasted) => {
                let wasteds = self.wasteds.entry(wasted.clone()).or_default();
                *wasteds += 1;

                let counts_at_len = self
                    .wasted_per_len
                    .entry(meta.found_paths.len())
                    .or_default();
                *counts_at_len += 1;
            }
            FailingCause::AStar(_) => {}
        }

        for (path, route) in meta.found_paths.iter().zip(meta.sequence.routes()) {
            let seen_mine = self.seen_mines.0.entry(route.location).or_default();
            *seen_mine += 1;

            let seen_destinations = self
                .seen_destinations
                .entry(path.segment.end.point())
                .or_default();
            *seen_destinations += 1;
        }
        self.last_meta = Some(meta);
    }

    fn is_more_failing_sequence(&self, other: &FailingMeta) -> bool {
        other.failing_sequence > self.best_meta.as_ref().unwrap().failing_sequence
    }

    fn last_best_meta(&mut self) {
        assert!(self.last_meta.is_some());
        self.best_meta = self.last_meta.take();
    }

    fn best_meta(&self) -> &FailingMeta<'plan_mine> {
        self.best_meta.as_ref().unwrap()
    }
}

impl Display for FailingStats<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let Self {
            last_meta: _,
            best_meta: _,
            seen_mines,
            seen_destinations,
            wasted_per_len,
            failures_per_len,
            wasteds,
        } = self;

        fn print_map(
            f: &mut Formatter<'_>,
            map: &HashMap<impl Debug + Ord + Hash, usize>,
        ) -> std::fmt::Result {
            for key in map.keys().sorted() {
                let count = map[key];
                writeln!(f, "{count:>4}  {key:?}")?;
            }
            Ok(())
        }

        writeln!(f, "\n=============\nFAILURE_STATS\n")?;

        writeln!(f, "- Seen Mines - ")?;
        print_map(f, &seen_mines.0)?;

        writeln!(f, "- Seen destinations - ")?;
        print_map(f, seen_destinations)?;

        writeln!(f, "- Wasted per len - ")?;
        print_map(f, wasted_per_len)?;

        writeln!(f, "- Wasted blocks - ")?;
        for (_, count) in wasteds {
            writeln!(f, "block used {count:>4}")?;
        }

        writeln!(f, "- Failures per len - ")?;
        print_map(f, failures_per_len)?;

        Ok(())
    }
}

mod _hidden_sequence {
    use crate::navigator::mine_executor::{ExecutionRoute, FailingMeta};

    #[repr(transparent)]
    #[derive(PartialEq, PartialOrd)]
    pub struct FailingSequence(usize);

    impl FailingSequence {
        pub(super) fn new(v: usize) -> Self {
            Self(v)
        }

        pub fn get_sequence_route<'plan_mine, 'meta>(
            &self,
            meta: &'meta FailingMeta<'plan_mine>,
        ) -> &'meta ExecutionRoute<'plan_mine> {
            meta.sequence.routes().get(self.0).unwrap()
        }

        pub fn split_at<'t, T>(&self, input: &'t [T]) -> (&'t [T], &'t [T]) {
            input.split_at(self.0)
        }
    }
}
use crate::navigator::base_source::BaseSourceEntry;
pub use _hidden_sequence::FailingSequence;

pub struct SeenMines<'plan_mine>(HashMap<&'plan_mine MineLocation, usize>);

impl<'plan_mine> SeenMines<'plan_mine> {
    pub fn least_known(&self) -> &'plan_mine MineLocation {
        self.0.iter().min_by_key(|(_, count)| *count).unwrap().0
    }

    pub fn mines(&self) -> impl Iterator<Item = &'plan_mine MineLocation> {
        self.0.keys().map(|v| *v)
    }

    pub fn counts(&self) -> std::collections::hash_map::Values<'_, &MineLocation, usize> {
        self.0.values()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

#[derive(PartialEq)]
pub enum ExecuteFlags {
    ShrinkBases,
}
