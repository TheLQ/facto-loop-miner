use crate::navigator::mori::{mori2_start, MoriResult};
use crate::surfacev::mine::{MineLocation, MinePath};
use crate::surfacev::vsurface::VSurface;
use facto_loop_miner_fac_engine::common::varea::VArea;
use facto_loop_miner_fac_engine::common::vpoint_direction::VSegment;
use facto_loop_miner_fac_engine::game_blocks::rail_hope_single::HopeLink;
use itertools::Itertools;
use rayon::prelude::*;
use rayon::ThreadPool;
use simd_json::prelude::ArrayTrait;
use std::cell::LazyCell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::ops::Range;
use std::sync::LazyLock;
use tracing::info;

/// A pure multi-executor. Input is final points
pub fn mori_boss(
    mode: BossMode,
    surface: &VSurface,
    routes_batches: Vec<Vec<BossRoute>>,
    finding_limiter: &VArea,
) -> Result<(usize, Vec<MinePath>, std::range::RangeInclusive<u32>), (HashMap<MineLocation, usize>)>
{
    assert!(!routes_batches.is_empty());
    for (i, batch) in routes_batches.iter().enumerate() {
        assert!(!batch.is_empty(), "empty {i} of {}", routes_batches.len());
    }

    let results: Vec<(usize, Result<Vec<MinePath>, MineLocation>)> = match mode {
        BossMode::Sequential => routes_batches
            .into_iter()
            .map(|batch| execute_batch(surface, batch, finding_limiter))
            .enumerate()
            .collect_vec(),
        BossMode::Threaded => {
            static THREAD_POOL: LazyLock<ThreadPool> = LazyLock::new(|| {
                let default_threads = rayon::current_num_threads();
                const THREAD_OVERSUBSCRIBE_PERCENT: f32 = 1.5;
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
            THREAD_POOL.install(|| {
                routes_batches
                    .into_par_iter()
                    .map(|batch| execute_batch(surface, batch, finding_limiter))
                    .enumerate()
                    .collect()
            })
        }
    };

    results
        .into_iter()
        // Find best path OR collect all the failed MineLocation's
        .fold(Err(HashMap::new()), |best, (i, batch_result)| {
            match (best, batch_result) {
                (Err(_), Ok(paths)) => {
                    // first good result
                    let total_cost = paths.iter().map(|v| v.cost).sum();
                    Ok((i, paths, (total_cost..=total_cost).into()))
                }
                (Ok((best_i, best_path, cost_range)), Ok(paths)) => {
                    // maybe improve best
                    let total_cost: u32 = paths.iter().map(|v| v.cost).sum();
                    match total_cost.cmp(&cost_range.start) {
                        Ordering::Less => {
                            // new lower cost
                            Ok((i, paths, (total_cost..=cost_range.end).into()))
                        }
                        Ordering::Equal => {
                            todo!("this happens?")
                        }
                        Ordering::Greater => {
                            // worse than best
                            Ok((
                                best_i,
                                best_path,
                                (cost_range.start..=total_cost.max(cost_range.end)).into(),
                            ))
                        }
                    }
                }
                //
                (Err(mut error_counters), Err(mine)) => {
                    // errors upon errors...
                    let counter = error_counters.entry(mine).or_insert(0);
                    *counter += 1;
                    Err(error_counters)
                }
                (Ok(best), Err(_mine)) => {
                    // ignore errors once we found a success
                    Ok(best)
                }
            }
        })
}

fn execute_batch(
    surface: &VSurface,
    batch: Vec<BossRoute>,
    finding_limiter: &VArea,
) -> Result<Vec<MinePath>, MineLocation> {
    let is_writable = batch.len() != 1;
    let mut working_surface: Option<VSurface> = None;

    let mut success_routes = Vec::new();

    let batch_len = batch.len();
    let mut batch_iter = batch.into_iter();
    while let Some(BossRoute(mine, segment)) = batch_iter.next() {
        let mori_res = mori2_start(
            working_surface.as_ref().map_or_else(|| surface, |v| v),
            segment.clone(),
            finding_limiter,
        );
        match mori_res {
            MoriResult::Route { path, cost } => {
                let mine_result = MinePath {
                    cost,
                    links: path,
                    mine_base: mine,
                    segment,
                };

                if is_writable {
                    let working_surface = working_surface.get_or_insert_with(|| surface.clone());
                    working_surface.add_mine_path(mine_result.clone()).unwrap()
                }

                success_routes.push(mine_result)
            }
            MoriResult::FailingDebug { .. } => {
                // todo: collect remaining routes?
                return Err(mine);
            }
        }
    }
    assert_eq!(success_routes.len(), batch_len);
    Ok(success_routes)
}

pub struct BossRoute(pub MineLocation, pub VSegment);

pub enum BossMode {
    Sequential,
    Threaded,
}
