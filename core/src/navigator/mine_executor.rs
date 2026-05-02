use crate::navigator::BaseSourceEighth;
use crate::navigator::mori::{LinkByImpl, MoriResult, mori2_start};
use crate::state::tuneables::MoriTunables;
use crate::surfacev::mine::{MineDraw, MineLocation, MineLocationResolver, MinePath};
use crate::surfacev::vsurface::{
    MineDestinationRef, MineRef, VSurfacePixel, VSurfacePixelAsVs, VSurfacePixelAsVsMut,
    VSurfacePixelMut, VSurfaceRail, VSurfaceRailAsVsMut,
};
use facto_loop_miner_common::duration::BasicWatch;
use facto_loop_miner_common::{EXECUTOR_TAG, LOCALE};
use facto_loop_miner_fac_engine::common::varea::VArea;
use facto_loop_miner_fac_engine::common::vpoint::VPoint;
use facto_loop_miner_fac_engine::common::vpoint_direction::VSegment;
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
use tracing::{Level, info, span};

pub fn execute_route_batch_clone_prep(
    tunables: &MoriTunables,
    surface: &mut VSurfacePixelMut,
    sequences: Vec<ExecutionSequence>,
    base_source: &BaseSourceEighth,
    mine_resolver: Vec<(MineRef, &MineLocation)>,
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

    execute_route_batch(
        tunables,
        surface.pixels(),
        sequences,
        base_source,
        mine_resolver,
        flags,
    )
}

/// Given thousands of possible route combinations, execute in parallel and find the best
pub fn execute_route_batch(
    tunables: &MoriTunables,
    execution_surface: VSurfacePixel,
    sequences: Vec<ExecutionSequence>,
    base_source: &BaseSourceEighth,
    mine_resolver: Vec<(MineRef, &MineLocation)>,
    flags: &[ExecuteFlags],
) -> ExecutorResult {
    let total_sequences = sequences.len();
    let unique_mines = {
        let mut seen: Vec<MineRef> = Vec::new();
        for sequence in &sequences {
            for route in sequence.routes() {
                let mine = route.destination.mine_ref();
                if !seen.contains(&mine) {
                    seen.push(mine);
                }
            }
        }
        seen.len()
    };

    // dedupe is bad
    {
        let mut seq_segments: Vec<Vec<MineDestinationRef>> = sequences
            .iter()
            .map(|v| v.routes().iter().map(|v| v.destination).collect())
            .collect();
        seq_segments.sort();
        seq_segments.dedup();

        assert_eq!(
            sequences.len(),
            total_sequences,
            "dedupe detected {}",
            sequences
                .iter()
                .map(|v| v
                    .routes()
                    .iter()
                    .map(|route| format!("{:?}", route.destination))
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
        // let default_threads = 32; // todo: numa rayon::current_num_threads();
        let default_threads = 24; // todo: numa rayon::current_num_threads();
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
                        &mine_resolver,
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
                    &mine_resolver,
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

fn execute_sequence(
    tuneables: &MoriTunables,
    surface_init: VSurfacePixel,
    sequence: ExecutionSequence,
    base_source: &BaseSourceEighth,
    mine_resolver: &[(MineRef, &MineLocation)],
    total_sequences: usize,
    flags: &[ExecuteFlags],
) -> ExecutorThreadResult {
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
    let mut surface = VSurfaceRail::surface_copy_no_rails(surface_init);
    // info!("Cloned surface in {}", watch);

    for (i, route) in sequence.routes().iter().enumerate() {
        if flags.contains(&ExecuteFlags::ShrinkBases) {
            MineDraw::ChangeNoTouch.draw_mine(
                &mut surface.pixels_mut_ref(),
                route
                    .destination
                    .mine_ref()
                    .resolve_mine_lookup(mine_resolver),
            );

            if i != 0 {
                MineDraw::ChangeBuffered.draw_mine(
                    &mut surface.pixels_mut_ref(),
                    sequence.routes()[i - 1]
                        .destination
                        .mine_ref()
                        .resolve_mine_lookup(mine_resolver),
                );
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
        let segment =
            route.segment_for_source(&source, MineLocationResolver::Lookup(mine_resolver));
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
                    destination: route.destination,
                    segment,
                };
                surface.rails_mut_ref().add_mine_path(path, "mori success");
            }
            MoriResult::FailingDebug { cause } => {
                FAIL_COUNTER.fetch_add(1, Ordering::Relaxed);
                return ExecutorThreadResult::Failure {
                    meta: FailingMeta {
                        sequence,
                        failing_sequence: FailingSequence::new(i),
                        cause,
                        found_paths: surface.rails_mut_ref().take_all_rails(),
                    },
                };
            }
        }
    }

    SUCCESS_COUNTER.fetch_add(1, Ordering::Relaxed);
    ExecutorThreadResult::Success {
        paths: surface.rails_mut_ref().take_all_rails(),
        sequence,
    }
}

pub struct ExecutionRoute {
    pub destination: MineDestinationRef,
    pub finding_limiter: VArea,
}

impl ExecutionRoute {
    pub fn segment_for_source(
        &self,
        source: &BaseSourceEntry,
        mine_resolver: MineLocationResolver,
    ) -> VSegment {
        let destination = mine_resolver.resolve_destination(self.destination);
        source.segment_for_mine(destination)
    }
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

    pub fn split_routes_from(
        &self,
        failing_sequence: FailingSequence,
    ) -> ExecutionSequenceParts<'_> {
        let (pass, fail) = failing_sequence.split_at(self.0.as_slice());
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
        stats: FailingStats,
    },
}

pub enum ExecutorThreadResult {
    Success {
        paths: Vec<MinePath>,
        sequence: ExecutionSequence,
    },
    Failure {
        meta: FailingMeta,
    },
}

impl ExecutorThreadResult {
    fn into_main_result(self) -> ExecutorResult {
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
pub struct FailingMeta {
    pub found_paths: Vec<MinePath>,
    pub sequence: ExecutionSequence,
    pub failing_sequence: FailingSequence,
    pub cause: FailingCause,
}

pub enum FailingCause {
    AStar(AStarErr<LinkByImpl, u32>),
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

pub struct FailingStats {
    pub last_meta: Option<FailingMeta>,
    pub best_meta: Option<FailingMeta>,
    /// MineLocation is owned by best_meta and last_meta chain. Can't make a ref
    /// avoid further lifetype noise with MineLocation.to_string()
    pub seen_mines: SeenMines,
    pub seen_destinations: HashMap<VPoint, usize>,
    pub wasted_per_len: HashMap<usize, usize>,
    pub failures_per_len: HashMap<usize, usize>,
    pub wasteds: HashMap<Vec<VPoint>, usize>,
}

impl FailingStats {
    fn new(best_meta: FailingMeta) -> Self {
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

    fn absorb_meta(&mut self, meta: FailingMeta) {
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
            let seen_mine = self
                .seen_mines
                .0
                .entry(route.destination.mine_ref())
                .or_default();
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

    fn best_meta(&self) -> &FailingMeta {
        self.best_meta.as_ref().unwrap()
    }
}

impl Display for FailingStats {
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
        for count in wasteds.values() {
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

        pub fn get_sequence_route<'meta>(&self, meta: &'meta FailingMeta) -> &'meta ExecutionRoute {
            meta.sequence.routes().get(self.0).unwrap()
        }

        pub fn split_at<'t, T>(&self, input: &'t [T]) -> (&'t [T], &'t [T]) {
            input.split_at(self.0)
        }
    }
}
use crate::navigator::base_source::BaseSourceEntry;
pub use _hidden_sequence::FailingSequence;

pub struct SeenMines(HashMap<MineRef, usize>);

impl SeenMines {
    pub fn least_known(&self) -> Option<MineRef> {
        self.0
            .iter()
            .min_by_key(|(_, count)| *count)
            .map(|(mine, _)| *mine)
    }

    pub fn mines(&self) -> impl Iterator<Item = MineRef> {
        self.0.keys().cloned()
    }

    pub fn counts(&self) -> impl Iterator<Item = usize> {
        self.0.values().cloned()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

#[derive(PartialEq)]
pub enum ExecuteFlags {
    ShrinkBases,
}
