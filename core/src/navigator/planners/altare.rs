use crate::navigator::base_source::BaseSourceEntry;
use crate::navigator::mine_permutate::{
    get_possible_routes_for_batch, CompletePlan, PlannedRoute, PlannedSequence,
};
use crate::navigator::mine_selector::{select_mines_and_sources, MineSelectBatch};
use crate::navigator::mori::{mori2_start, MoriResult};
use crate::navigator::planners::common::{
    debug_draw_failing_mines, draw_active_no_touching_zone, draw_no_touching_zone,
    draw_restored_no_touching_zone,
};
use crate::state::machine::StepParams;
use crate::surface::pixel::Pixel;
use crate::surfacev::mine::{MineLocation, MinePath};
use crate::surfacev::sanity::assert_sanity_mines_not_deduped;
use crate::surfacev::vsurface::VSurface;
use facto_loop_miner_fac_engine::common::vpoint_direction::VSegment;
use facto_loop_miner_fac_engine::util::ansi::{ansi_color, Color};
use itertools::Itertools;
use rayon::prelude::*;
use rayon::ThreadPool;
use simd_json::prelude::ArrayTrait;
use std::cmp::PartialEq;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::mem;
use std::ops::ControlFlow;
use tracing::{debug, error, info, warn};

/// Planner v2 "Regis Altare 🎇"
///
/// Advanced perfecting backtrack algorithm,
/// because v0 and v1 Ruze Planner can mask valid     
pub fn start_altare_planner(surface: &mut VSurface) {
    let exe_pool = rayon::ThreadPoolBuilder::new()
        .thread_name(|i| format!("exe{i:02}"))
        .num_threads(2)
        .build()
        .unwrap();

    let mut winder = Winder::new(surface);

    draw_no_touching_zone(surface, &winder.mines_remaining);

    let mut is_rewinding = false;
    while let Some(select) = winder.next_select() {
        assert_eq!(select.mines.len(), 1);
        let prev_no_touch = draw_active_no_touching_zone(surface, &select.mines[0]);

        let process = process_select(surface, &select, &exe_pool);
        match winder.apply(surface, select, process) {
            WinderNext::Continue => {}
            res => {
                error!("{res}, stop");
                break;
            }
        }

        draw_restored_no_touching_zone(surface, prev_no_touch);
    }
}

fn process_select(
    surface: &VSurface,
    select: &MineSelectBatch,
    exe_pool: &ThreadPool,
) -> Result<MinePath, Vec<PlannedRoute>> {
    assert_eq!(select.mines.len(), 1);
    info!("processing {:?}", select.mines[0]);

    let CompletePlan {
        sequences,
        base_sources,
    } = get_possible_routes_for_batch(surface, select.clone());
    // assert!(
    //     sequences.len() <= 2,
    //     "too many destinations {}?",
    //     sequences.len()
    // );
    // assert_ne!(sequences.len(), 0, "no destinations found?");
    assert_eq!(sequences.len(), 2, "not enough destinations found?");

    let actual_base_source = base_sources.borrow().peek_single();
    let mut results: Vec<(MoriResult, PlannedRoute)> = exe_pool.install(|| {
        sequences
            .into_par_iter()
            .map(|PlannedSequence { mut routes }| {
                assert_eq!(routes.len(), 1, "sequence should go to 1 route");
                let route = routes.remove(0);
                let res = execute_route(surface, &route, &actual_base_source);
                (res, route)
            })
            .collect()
    });

    results
        .into_iter()
        // Find best path OR collect all the failed MineLocation's
        .fold(Err(Vec::new()), |best, (res, route)| match (best, res) {
            (Err(_), MoriResult::Route { path, cost }) => Ok((path, cost, route)),
            (Ok(best), MoriResult::Route { path, cost }) => {
                if best.1 < cost {
                    Ok(best)
                } else {
                    Ok((path, cost, route))
                }
            }
            //
            (Err(mut total), MoriResult::FailingDebug(_, _)) => {
                total.push(route);
                Err(total)
            }
            (Ok(best), MoriResult::FailingDebug(_, _)) => Ok(best),
        })
        .map(|(links, cost, route)| {
            base_sources.borrow_mut().next().unwrap();

            MinePath {
                links,
                cost,
                mine_base: route.location,
            }
        })
        .inspect_err(|routes| assert!(!routes.is_empty()))
}

fn execute_route(
    surface: &VSurface,
    route: &PlannedRoute,
    base_source_entry: &BaseSourceEntry,
) -> MoriResult {
    // let mut working_surface = (*surface).clone();
    let working_surface = surface;

    let route_result = mori2_start(
        &working_surface,
        base_source_entry.route_to_segment(route),
        &route.finding_limiter,
    );
    route_result
}

enum WinderState {
    Normal,
    Rewinding {
        cause: Option<MineSelectBatch>,
        remaining: Vec<MineSelectBatch>,
        processed: Vec<(MineSelectBatch, MinePath)>,
    },
}

const IMPOSSIBLE_TRIGGER: usize = 3;

/// Wind and Re-Wind state
struct Winder {
    state: WinderState,
    mines_remaining: Vec<MineSelectBatch>,
    mines_processed: Vec<(MineSelectBatch, MinePath)>,
    impossibles: HashMap<MineLocation, usize>,
    impossible_for_remaining: usize,
}

impl Winder {
    fn new(surface: &VSurface) -> Self {
        let mut mines_remaining = select_mines_and_sources(&surface, 1)
            .into_success()
            .unwrap();
        // to use push-pop semantics
        mines_remaining.reverse();
        assert_sanity_mines_not_deduped(mines_remaining.iter().map(|v| &v.mines[0]));
        Self {
            state: WinderState::Normal,
            mines_remaining,
            mines_processed: Vec::new(),
            impossibles: HashMap::new(),
            impossible_for_remaining: usize::MAX,
        }
    }

    fn next_select(&mut self) -> Option<MineSelectBatch> {
        let mut debug_rewinding = "".to_string();
        let res = if let WinderState::Rewinding {
            cause,
            remaining,
            processed,
        } = &mut self.state
        {
            debug_rewinding = format!(
                " | rewind_remaining {:>3} rewind_processed {:>3}",
                remaining.len(),
                processed.len()
            );
            let next = cause.clone().or_else(|| remaining.pop());
            // SAFETY: this state should have something
            Some(next.unwrap())
        } else {
            self.mines_remaining.pop()
        };

        info!(
            "{}",
            ansi_color(
                format!(
                    "remaining {:>3} processed {:>3} state {}{debug_rewinding}",
                    self.mines_remaining.len(),
                    self.mines_processed.len(),
                    self.state.name()
                ),
                Color::BrightCyan
            )
        );
        res
    }

    fn apply(
        &mut self,
        surface: &mut VSurface,
        select: MineSelectBatch,
        result: Result<MinePath, Vec<PlannedRoute>>,
    ) -> WinderNext {
        // fn rewind(winder: &mut Winder, surface: &mut VSurface) -> WinderNext {
        //     let Some((last_select, path)) = winder.mines_processed.pop() else {
        //         return WinderNext::BreakNoMoreProcessed;
        //     };
        //     surface.remove_mine_path(&path);
        //     winder.mines_rewinded_remaining.push(last_select);
        //     WinderNext::Continue
        // }

        let mut intra_state = WinderState::Normal;
        mem::swap(&mut self.state, &mut intra_state);
        let (new_state, next) = match (intra_state, result) {
            // all is good, apply normally
            (WinderState::Normal, Ok(mine_path)) => {
                debug!("HMM: Normal, normal");
                self.mines_processed
                    .push((select.clone(), mine_path.clone()));
                surface.add_mine_path(mine_path).unwrap();
                (WinderState::Normal, WinderNext::Continue)
            }
            // was normal now error, start backtracing
            (WinderState::Normal, Err(_debug)) => {
                debug!("HMM: Normal, failed");
                let Some((last_select, path)) = self.mines_processed.pop() else {
                    return WinderNext::BreakNoMoreProcessed;
                };
                surface.remove_mine_path(&path);

                let state = WinderState::Rewinding {
                    cause: Some(select),
                    remaining: vec![last_select],
                    processed: Vec::new(),
                };
                (state, WinderNext::Continue)
            }
            // Recovery success, either middle step or end step
            (
                WinderState::Rewinding {
                    cause,
                    remaining,
                    mut processed,
                },
                Ok(mine_path),
            ) => {
                debug!("HMM: Rewinding, success");
                surface.add_mine_path(mine_path.clone()).unwrap();
                if let Some(cause) = cause {
                    assert_eq!(cause.mines, select.mines);
                    // we fixed cause
                    processed.push((cause, mine_path));
                } else {
                    // we processed a remaining?
                    processed.push((select, mine_path));
                };
                assert_sanity_mines_not_deduped(remaining.iter().map(|v| &v.mines[0]));

                if remaining.is_empty() {
                    // nothing to do anymore
                    self.mines_processed.append(&mut processed);
                    (WinderState::Normal, WinderNext::Continue)
                } else {
                    // still in recovery
                    (
                        WinderState::Rewinding {
                            // grabbed the cause already
                            cause: None,
                            remaining,
                            processed,
                        },
                        WinderNext::Continue,
                    )
                }
            }
            // while rewinding we still failed, go back more
            (
                WinderState::Rewinding {
                    cause,
                    mut remaining,
                    processed,
                },
                Err(_routes),
            ) => {
                debug!("HMM: Rewinding, failed again");
                if let Some(cause) = cause {
                    assert_eq!(cause.mines, select.mines);
                    // do nothing else, this is later re-added as the cause
                }
                // throw away rewind results
                for (select, path) in processed.into_iter() {
                    remaining.push(select);
                    surface.remove_mine_path(&path);
                }
                // try removing something in the way
                // todo: or loop...
                if let Some((last_select, path)) = self.mines_processed.pop() {
                    remaining.push(last_select);
                    surface.remove_mine_path(&path);
                }

                if remaining.is_empty() {
                    return WinderNext::BreakRewindingFailed;
                }

                // DEBUG SANITY CHECKING
                assert_sanity_mines_not_deduped(remaining.iter().map(|v| &v.mines[0]));

                // DEBUG SANITY CHECKING
                // remaining.push(select);
                let all_mine_bases = remaining
                    .iter()
                    .chain(self.mines_remaining.iter())
                    .flat_map(|v| {
                        assert_eq!(v.mines.len(), 1);
                        v.mines[0].patch_indexes.clone()
                    })
                    .collect_vec();
                // let select = remaining.pop().unwrap();
                let any_exist = surface.get_mine_paths().iter().find(|v| {
                    v.mine_base
                        .patch_indexes
                        .iter()
                        .any(|patch| all_mine_bases.contains(patch))
                });
                assert_eq!(any_exist, None, "remaining found in surface");

                // loop detection
                if self.impossible_for_remaining != self.mines_remaining.len() {
                    self.impossible_for_remaining = self.mines_remaining.len();
                    self.impossibles.clear();
                }
                let history = *self
                    .impossibles
                    .entry(select.mines[0].clone())
                    .and_modify(|v| *v += 1)
                    .or_insert(1);
                let cause = if history == IMPOSSIBLE_TRIGGER {
                    // skippa skippa
                    warn!("HMM: purging impossible {:?}", select.mines[0]);
                    Some(remaining.pop().unwrap())
                } else {
                    Some(select)
                };

                (
                    WinderState::Rewinding {
                        // existing cause was moved already
                        cause,
                        remaining,
                        // moved to remaining
                        processed: Vec::new(),
                    },
                    WinderNext::Continue,
                )
            }
        };
        self.state = new_state;
        next
    }
}

impl WinderState {
    fn name(&self) -> &str {
        match self {
            Self::Normal => "Normal",
            Self::Rewinding { .. } => "Rewinding",
            // Self::Recovery => "Recovery",
        }
    }
}

enum WinderNext {
    Continue,
    BreakNoMoreProcessed,
    BreakRewindingFailed,
}

impl Display for WinderNext {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::Continue => "Continue",
            Self::BreakNoMoreProcessed => "BreakNoMoreProcessed",
            Self::BreakRewindingFailed => "BreakRewindingFailed",
        };
        f.write_str(name)
    }
}
