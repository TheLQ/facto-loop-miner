use crate::navigator::base_source::{BaseSourceEighth, BaseSourceEntry};
use crate::navigator::mine_permutate::{get_possible_routes_for_batch, CompletePlan};
use crate::navigator::mine_selector::{select_mines_and_sources, MineSelectBatch};
use crate::navigator::mori::{mori2_start, MoriResult};
use crate::navigator::mori_boss::{mori_boss, BossMode, BossRoute};
use crate::navigator::planners::common::draw_prep_mines;
use crate::surface::pixel::Pixel;
use crate::surfacev::mine::{MineLocation, MinePath};
use crate::surfacev::sanity::assert_sanity_mines_not_deduped;
use crate::surfacev::vsurface::VSurface;
use facto_loop_miner_fac_engine::common::varea::VArea;
use facto_loop_miner_fac_engine::common::vpoint::{VPoint, VPOINT_ZERO};
use facto_loop_miner_fac_engine::common::vpoint_direction::{VPointDirectionQ, VSegment};
use facto_loop_miner_fac_engine::game_blocks::rail_hope::RailHopeLink;
use facto_loop_miner_fac_engine::game_blocks::rail_hope_single::SECTION_POINTS_I32;
use facto_loop_miner_fac_engine::util::ansi::{ansi_color, Color};
use itertools::Itertools;
use rayon::prelude::*;
use rayon::ThreadPool;
use simd_json::prelude::ArrayTrait;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::mem;
use std::rc::Rc;
use tracing::{debug, error, info, warn};

/// Planner v2 "Regis Altare 🎇"
///
/// Advanced perfecting backtrack algorithm,
/// because v0 Mori and v1 Ruze Planner can mask valid routes
pub fn start_altare_planner(surface: &mut VSurface) {
    let mut winder = Winder::new(surface);

    draw_prep_mines(
        surface,
        winder.mines_remaining.iter().map(|v| &v.mine),
        &winder.mines_remaining[0].base_sources,
    );
    // surface.validate();

    let dummy_finding_limiter =
        VArea::from_arbitrary_points_pair(surface.point_top_left(), surface.point_bottom_right());

    while let Some(select) = winder.next_select() {
        let prev_no_touch = select.mine.draw_area_buffered_to_no_touch(surface);

        let process = process_select(surface, &select, &dummy_finding_limiter);
        match winder.apply(surface, select.clone(), process) {
            WinderNext::Continue => {}
            breaker => {
                let res = match breaker {
                    WinderNext::Continue => unreachable!(),
                    WinderNext::BreakRewindingFailed => "BreakRewindingFailed",
                    WinderNext::BreakNoMoreProcessed => "BreakNoMoreProcessed",
                };

                for endpoint in select.mine.destinations() {
                    let route = PlannedRoute {
                        location: select.mine.clone(),
                        destination: *endpoint,
                        finding_limiter: VArea::from_arbitrary_points([VPOINT_ZERO]),
                    };
                    let route = select
                        .base_sources
                        .borrow()
                        .peek_single()
                        .route_to_segment(&route);
                    if select.mine.destinations().contains(&route.end) {
                        warn!("destination and end match")
                    }
                    surface
                        .set_pixels(
                            Pixel::Highlighter,
                            vec![*route.start.point(), *route.end.point()],
                        )
                        .unwrap();
                }

                surface.set_entity_replace(
                    select.mine.area_no_touch().point_top_left(),
                    Pixel::MineNoTouch,
                    Pixel::EdgeWall,
                );

                surface
                    .set_pixels(Pixel::SteelChest, select.mine.endpoints().to_vec())
                    .unwrap();
                surface.save_pixel_to_oculante();
                error!("{res}, stop, drawing");
                break;
            }
        }

        surface.save_pixel_to_oculante();
        // if winder.mines_processed.len() == 8 {
        //     break;
        // }
        MineLocation::draw_area_no_touch_to_buffered(surface, prev_no_touch);

        // let mut is_clean_run = matches!(winder.state, WinderState::Normal);
        // match winder.reorder_processing(surface, select) {
        //     ReorderResult::Continue => {}
        //     ReorderResult::BackoutAndRetry(location) => {
        //         is_clean_run = false;
        //         let mine_path_index = surface
        //             .get_mine_paths()
        //             .iter()
        //             .position(|p| p.mine_base == location)
        //             .unwrap();
        //         surface.remove_mine_path_at_index(mine_path_index);
        //     }
        //     ReorderResult::BreakNotNormal => {
        //         is_clean_run = false;
        //         error!("BreakNotNormal, stop reorder");
        //         break;
        //     }
        // }

        // if is_clean_run {
        optimize_latest_processed(&mut winder, surface, &dummy_finding_limiter);
        // }

        if matches!(winder.state, WinderState::Normal) && winder.mines_processed.len() > 50 {
            error!("ENDING");
            break;
        }
    }
}

fn process_select(
    surface: &VSurface,
    select: &SlimeSelect,
    finding_limiter: &VArea,
) -> Result<MinePath, Vec<MineLocation>> {
    // info!("processing {:?}", select.mines[0]);

    let actual_base_source = select.base_sources.borrow().peek_single();
    let route_batches = select
        .to_routes(surface)
        .map(|route| {
            let segment = actual_base_source.route_to_segment(&route);
            // vec because 1 route per batch
            vec![BossRoute(route.location, segment)]
        })
        .collect_vec();

    let boss_result = mori_boss(
        BossMode::Sequential,
        surface,
        route_batches,
        &finding_limiter,
    );
    match boss_result {
        Ok((i, mut best_path, cost_range)) => {
            assert_eq!(best_path.len(), 1);
            let best_path = best_path.remove(0);

            select.base_sources.borrow_mut().next().unwrap();
            // info!(
            //     "found best path with {} links range {cost_range}",
            //     best_path
            // );
            Ok(best_path)
        }
        Err(most_failed_mines) => Err(most_failed_mines.into_keys().collect_vec()),
    }
    .inspect_err(|routes| assert!(!routes.is_empty()))
}

fn optimize_latest_processed(winder: &mut Winder, surface: &mut VSurface, finding_limiter: &VArea) {
    const CHUNK_SIZE: usize = 2;

    let needle_base_sources = winder
        .mines_processed
        .last()
        .unwrap()
        .select
        .base_sources
        .clone();
    let to_optimize_pos = winder
        .mines_processed
        .iter()
        .enumerate()
        .filter(|(i, processed)| processed.select.base_sources == needle_base_sources)
        .map(|(i, _)| i)
        .take(CHUNK_SIZE)
        .collect_vec();
    if to_optimize_pos.len() != CHUNK_SIZE {
        debug!(
            "not enough mines {} in same base_sources to optimize",
            to_optimize_pos.len()
        );
        return;
    }

    info!("opmizing------");
    let mut to_optimize = Vec::new();
    for pos in to_optimize_pos.into_iter().rev() {
        to_optimize.push(winder.mines_processed.remove(pos))
    }
    assert_eq!(to_optimize.len(), CHUNK_SIZE);

    let mut new_starts_base = needle_base_sources
        .borrow()
        .peek_multiple_backwards(CHUNK_SIZE);

    // remove from surface, get cost, extract endpoints
    let mut orig_total_cost = 0;
    let mut new_starts_extracted: Vec<VPointDirectionQ> = Vec::new();
    let mut new_ends: Vec<(MineLocation, VPointDirectionQ)> = Vec::new();
    for to_optimize_processed in &to_optimize {
        let index = surface
            .get_mine_paths()
            .iter()
            .position(|v| v == to_optimize_processed.path)
            .unwrap();
        // had ref now owned
        let path = surface.remove_mine_path_at_index(index);

        orig_total_cost += path.cost;

        // detect which one we are
        let my_base_index = new_starts_base
            .iter()
            .position(|base| base.origin == path.segment.start)
            .unwrap();
        let my_base = new_starts_base.remove(my_base_index);

        new_starts_extracted.push(path.segment.start);

        new_ends.push((path.mine_base, path.segment.end));
    }

    // get possibilities of endpoints and make them
    let permutations = new_ends.len();
    let boss_routes: Vec<Vec<BossRoute>> = new_ends
        .into_iter()
        .permutations(permutations)
        .map(|ends| {
            ends.into_iter()
                .enumerate()
                .map(|(i, (mine, end))| {
                    BossRoute(
                        mine,
                        VSegment {
                            start: new_starts[i].clone(),
                            end,
                        },
                    )
                })
                .collect_vec()
        })
        .collect_vec();
    info!("optimizer: generated {} boss_routes", boss_routes.len());

    // pathfind!
    let (
        best_paths,
        std::range::RangeInclusive {
            start: new_total_cost,
            end: _,
        },
    ) = mori_boss(BossMode::Sequential, surface, boss_routes, &finding_limiter)
        // at least one of these routes worked before
        .unwrap();

    assert!(!best_paths.is_empty());
    info!(
        "Optimizer reduced cost from {} to {} diff {}",
        orig_total_cost,
        new_total_cost,
        orig_total_cost - new_total_cost
    );
    if orig_total_cost - new_total_cost == 0 {
        panic!("didn't improve??")
    }
    // restore surface
    for path in best_paths {
        surface.add_mine_path(path).unwrap()
    }
}

#[derive(Clone)]
struct SlimeSelect {
    pub mine: MineLocation,
    pub base_sources: Rc<RefCell<BaseSourceEighth>>,
}

impl From<MineSelectBatch> for SlimeSelect {
    fn from(
        MineSelectBatch {
            mut mines,
            base_sources,
        }: MineSelectBatch,
    ) -> Self {
        assert_eq!(mines.len(), 1);
        Self {
            mine: mines.remove(0),
            base_sources,
        }
    }
}

impl SlimeSelect {
    fn to_routes<'a>(
        &self,
        surface: &'a VSurface,
    ) -> impl Iterator<Item = PlannedRoute> + use<'_, 'a> {
        self.mine
            .destinations()
            .iter()
            .map(|destination| PlannedRoute {
                destination: *destination,
                location: self.mine.clone(),
                finding_limiter: VArea::from_arbitrary_points_pair(
                    surface.point_top_left(),
                    surface.point_bottom_right(),
                ),
            })
    }
}

struct SliceProcessed {
    select: SlimeSelect,
    path: MinePath,
    applied_source: BaseSourceEighth,
    path_segment: VSegment,
}

enum WinderState {
    Normal,
    Rewinding {
        cause: Option<SlimeSelect>,
        remaining: Vec<SlimeSelect>,
        processed: Vec<()>,
    },
}

const IMPOSSIBLE_TRIGGER: usize = 3;

/// Wind and Re-Wind state
struct Winder {
    state: WinderState,
    mines_remaining: Vec<SlimeSelect>,
    mines_processed: Vec<SliceProcessed>,
    impossibles: HashMap<MineLocation, usize>,
    impossible_for_remaining: usize,
}

impl Winder {
    fn new(surface: &VSurface) -> Self {
        let mut mines_remaining: Vec<SlimeSelect> = select_mines_and_sources(&surface, 1)
            .into_success()
            .unwrap()
            .into_iter()
            .map(SlimeSelect::from)
            .collect();
        // to use push-pop semantics
        mines_remaining.reverse();
        assert_sanity_mines_not_deduped(mines_remaining.iter().map(|v| &v.mine));
        Self {
            state: WinderState::Normal,
            mines_remaining,
            mines_processed: Vec::new(),
            impossibles: HashMap::new(),
            impossible_for_remaining: usize::MAX,
        }
    }

    fn next_select(&mut self) -> Option<SlimeSelect> {
        let mut debug_rewinding = "".to_string();
        let res = if let WinderState::Rewinding {
            cause,
            remaining,
            processed,
        } = &mut self.state
        {
            debug_rewinding = format!(
                " | {:>3} rewind_remaining   {:>3} rewind_processed ",
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
                    "{:>3} remaining   {:>3} processed   {}{debug_rewinding}",
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
        select: SlimeSelect,
        result: Result<(MinePath, BaseSourceEighth, VSegment), Vec<MineLocation>>,
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
            (WinderState::Normal, Ok((mine_path, applied_source, path_segment))) => {
                debug!("HMM: Normal, normal");
                self.mines_processed.push(SliceProcessed {
                    path: mine_path.clone(),
                    select,
                    applied_source,
                    path_segment,
                });
                surface.add_mine_path(mine_path).unwrap();
                (WinderState::Normal, WinderNext::Continue)
            }
            // was normal now error, start backtracing
            (WinderState::Normal, Err(_debug)) => {
                debug!("HMM: Normal, failed");
                let Some((last_select, path)) = self.mines_processed.pop() else {
                    return WinderNext::BreakNoMoreProcessed;
                };
                // surface.remove_mine_path(&path);

                (
                    WinderState::Rewinding {
                        cause: Some(select),
                        remaining: vec![last_select],
                        processed: Vec::new(),
                    },
                    WinderNext::BreakRewindingFailed,
                )
            }
            (WinderState::Rewinding { .. }, _) => {
                unimplemented!()
            } /*
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
                      assert_eq!(cause.mine, select.mine);
                      // we fixed cause
                      processed.push((cause, mine_path));
                  } else {
                      // we processed a remaining?
                      processed.push((select, mine_path));
                  };
                  assert_sanity_mines_not_deduped(remaining.iter().map(|v| &v.mine));

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
                      assert_eq!(cause.mine, select.mine);
                      // do nothing else, this is later re-added as the cause
                  }

                  // try removing something in the way
                  // todo: or loop...
                  if let Some((last_select, path)) = self.mines_processed.pop() {
                      remaining.push(last_select);
                      // surface.remove_mine_path(&path);
                  }
                  // throw away rewind results
                  for (select, path) in processed.into_iter() {
                      remaining.push(select);
                      // surface.remove_mine_path(&path);
                  }

                  if remaining.is_empty() {
                      return WinderNext::BreakRewindingFailed;
                  }

                  // // DEBUG SANITY CHECKING
                  // assert_sanity_mines_not_deduped(remaining.iter().map(|v| &v.mines[0]));
                  //
                  // // DEBUG SANITY CHECKING
                  // // remaining.push(select);
                  // let all_mine_bases = remaining
                  //     .iter()
                  //     .chain(self.mines_remaining.iter())
                  //     .flat_map(|v| {
                  //         assert_eq!(v.mines.len(), 1);
                  //         v.mines[0].patch_indexes.clone()
                  //     })
                  //     .collect_vec();
                  // // let select = remaining.pop().unwrap();
                  // let any_exist = surface.get_mine_paths().iter().find(|v| {
                  //     v.mine_base
                  //         .patch_indexes
                  //         .iter()
                  //         .any(|patch| all_mine_bases.contains(patch))
                  // });
                  // assert_eq!(any_exist, None, "remaining found in surface");

                  // loop detection
                  // if self.impossible_for_remaining != self.mines_remaining.len() {
                  //     self.impossible_for_remaining = self.mines_remaining.len();
                  //     self.impossibles.clear();
                  // }
                  // let history = *self
                  //     .impossibles
                  //     .entry(select.mines[0].clone())
                  //     .and_modify(|v| *v += 1)
                  //     .or_insert(1);
                  // let cause = if history == IMPOSSIBLE_TRIGGER {
                  //     // skippa skippa
                  //     warn!("HMM: purging impossible {:?}", select.mines[0]);
                  //     self.impossibles.clear();
                  //     Some(remaining.pop().unwrap())
                  // } else {
                  //     Some(select)
                  // };
                  let cause = Some(select);

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
              }*/
        };
        self.state = new_state;
        next
    }

    fn reorder_processing(
        &mut self,
        surface: &VSurface,
        last_processed_mine: SlimeSelect,
    ) -> ReorderResult {
        if !matches!(self.state, WinderState::Normal) {
            error!("umm");
            return ReorderResult::BreakNotNormal;
        }

        // split remaining into
        //  - include horizontal band defined by height of processed mines
        //  - excluded
        let edge_start = *self
            .mines_remaining
            .last()
            .unwrap()
            .base_sources
            .borrow()
            .peek_single()
            .origin
            .point();
        let edge_end = VPoint::new(surface.get_radius_i32(), edge_start.y());
        error!("edge_start {edge_start}");
        error!("edge_end   {edge_end}");

        let remaining_before_len = self.mines_remaining.len();

        // Do splits
        let mut included = Vec::new();
        let mut excluded = Vec::new();
        let mut processed_area = VArea::from_arbitrary_points(
            self.mines_processed
                .iter()
                .flat_map(|(select, path)| &path.links)
                .map(|link| link.pos_next())
                .chain([edge_start, edge_end]),
        );
        for edge_expansion in 0.. {
            processed_area = VArea::from_arbitrary_points_pair(
                processed_area
                    .point_top_left()
                    .move_y(-SECTION_POINTS_I32 * edge_expansion),
                processed_area
                    .point_bottom_right()
                    .move_y(SECTION_POINTS_I32 * edge_expansion),
            );

            for select in mem::take(&mut self.mines_remaining) {
                if select
                    .mine
                    .area_buffered()
                    .get_corner_points()
                    .iter()
                    .any(|v| processed_area.contains_point(v))
                {
                    included.push(select);
                } else {
                    excluded.push(select);
                }
            }

            if included.is_empty() {
                // All results moved here, restore remaining for next iteration
                self.mines_remaining = mem::take(&mut excluded);
            } else {
                break;
            }
        }

        // check where our mine should have been in this new search area
        let last_processed_mine_location = last_processed_mine.mine.clone();
        included.push(last_processed_mine);

        // sort the included into preferred order
        included.sort_by(|a, b| {
            VPoint::sort_by_x_then_y_column(
                a.mine.area_buffered().point_top_left(),
                b.mine.area_buffered().point_top_left(),
            )
        });

        self.mines_remaining.extend(included);
        self.mines_remaining.extend(excluded);
        self.mines_remaining.reverse();
        assert_eq!(
            remaining_before_len + /*last mine check*/1,
            self.mines_remaining.len()
        );

        let last_advice = self
            .mines_remaining
            .last()
            .unwrap()
            .mine
            .area_buffered()
            .get_corner_points();
        assert!(
            last_advice.iter().any(|v| processed_area.contains_point(v)),
            "area {processed_area} last {}",
            last_advice.map(|v| v.to_string()).join(",")
        );

        if self.mines_remaining.last().unwrap().mine != last_processed_mine_location {
            ReorderResult::BackoutAndRetry(last_processed_mine_location)
        } else {
            self.mines_remaining.pop();
            ReorderResult::Continue
        }
    }

    fn steal_nearby_routes(&self) {
        // find the closest rail in VEntityMap
        // map back to it's MineLocation and endpoints
        // Remove from surface
        // set new destination to this, re-path

        todo!()
    }

    fn find_next_rail() {
        // Instead of precalculated order,
        // Pick the (farthest?) patch on x axis
        // Range is the distance between the connected patches, working (towards?) middle
        todo!()
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

enum ReorderResult {
    Continue,
    BackoutAndRetry(MineLocation),
    BreakNotNormal,
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
