use crate::navigator::base_source::{BaseSourceEighth, BaseSourceEntry};
use crate::navigator::mine_permutate::{
    get_possible_routes_for_batch, CompletePlan, PlannedRoute, PlannedSequence,
};
use crate::navigator::mine_selector::{select_mines_and_sources, MineSelectBatch};
use crate::navigator::mori::{mori2_start, MoriResult};
use crate::navigator::planners::common::draw_prep_mines;
use crate::surfacev::mine::{MineLocation, MinePath};
use crate::surfacev::sanity::assert_sanity_mines_not_deduped;
use crate::surfacev::vsurface::VSurface;
use facto_loop_miner_fac_engine::common::varea::VArea;
use facto_loop_miner_fac_engine::common::vpoint::VPoint;
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
    let exe_pool = rayon::ThreadPoolBuilder::new()
        .thread_name(|i| format!("exe{i:02}"))
        .num_threads(2)
        .build()
        .unwrap();

    let mut winder = Winder::new(surface);

    draw_prep_mines(
        surface,
        winder.mines_remaining.iter().map(|v| &v.mine),
        &winder.mines_remaining[0].base_sources,
    );
    // surface.validate();

    while let Some(select) = winder.next_select() {
        let prev_no_touch = select.mine.draw_area_buffered_to_no_touch(surface);

        let process = process_select(surface, &select, &exe_pool);
        match winder.apply(surface, select, process) {
            WinderNext::Continue => {}
            res => {
                error!("{res}, stop");
                break;
            }
        }

        surface.save_pixel_to_oculante();
        // if winder.mines_processed.len() == 8 {
        //     break;
        // }
        MineLocation::draw_area_no_touch_to_buffered(surface, prev_no_touch);

        match winder.reorder_processing(surface) {
            WinderNext::Continue => {}
            res => {
                error!("{res}, stop reorder");
                break;
            }
        }

        if matches!(winder.state, WinderState::Normal) && winder.mines_processed.len() > 50 {
            error!("ENDING");
            break;
        }
    }
}

fn process_select(
    surface: &VSurface,
    select: &SlimeSelect,
    exe_pool: &ThreadPool,
) -> Result<MinePath, Vec<PlannedRoute>> {
    // info!("processing {:?}", select.mines[0]);

    let actual_base_source = select.base_sources.borrow().peek_single();
    let routes = select.to_routes(surface).collect_vec();
    let results: Vec<(MoriResult, PlannedRoute)> = exe_pool.install(|| {
        routes
            .into_par_iter()
            .map(|route| {
                let res = execute_route(surface, &route, &actual_base_source);
                (res, route)
            })
            .collect()
    });
    assert_ne!(results.len(), 0, "no destinations found?");

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
            select.base_sources.borrow_mut().next().unwrap();

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
    // We run once per iteration so no mut, unlike Ruze that runs an entire batch
    let working_surface = surface;

    let route_result = mori2_start(
        &working_surface,
        base_source_entry.route_to_segment(route),
        &route.finding_limiter,
    );
    route_result
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

enum WinderState {
    Normal,
    Rewinding {
        cause: Option<SlimeSelect>,
        remaining: Vec<SlimeSelect>,
        processed: Vec<(SlimeSelect, MinePath)>,
    },
}

const IMPOSSIBLE_TRIGGER: usize = 3;

/// Wind and Re-Wind state
struct Winder {
    state: WinderState,
    mines_remaining: Vec<SlimeSelect>,
    mines_processed: Vec<(SlimeSelect, MinePath)>,
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
                self.mines_processed.push((select, mine_path.clone()));
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
                    WinderNext::Continue,
                )
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
            }
        };
        self.state = new_state;
        next
    }

    fn reorder_processing(&mut self, surface: &VSurface) -> WinderNext {
        if !matches!(self.state, WinderState::Normal) {
            error!("umm");
            return WinderNext::BreakRewindingFailed;
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
        let mut processed_area = VArea::from_arbitrary_points(
            self.mines_processed
                .iter()
                .flat_map(|(select, path)| select.mine.area_buffered().get_corner_points())
                .chain([edge_start, edge_end]),
        );
        error!("processed len {}", self.mines_processed.len());
        error!(
            "pos_start {}",
            self.mines_processed[0]
                .0
                .mine
                .area_buffered()
                .point_top_left()
        );
        error!("area {processed_area}");
        // panic!("uhh");
        let remaining_before_len = self.mines_remaining.len();
        let mut inner_remaining = mem::take(&mut self.mines_remaining);
        let mut included = Vec::new();
        let mut excluded = Vec::new();
        for select in inner_remaining {
            if select
                .mine
                .area_buffered()
                .get_corner_points()
                .iter()
                .any(|v| processed_area.contains_point(v))
            {
                // if processed_area.contains_point(&select.only_mine().area.point_top_left()) {
                included.push(select);
            } else {
                excluded.push(select);
            }
        }

        let mut expansion = 0;
        while included.is_empty() {
            expansion += 1;

            processed_area = VArea::from_arbitrary_points_pair(
                processed_area.point_top_left().move_y(-SECTION_POINTS_I32),
                processed_area
                    .point_bottom_right()
                    .move_y(SECTION_POINTS_I32),
            );
            warn!("expanding {expansion} with {processed_area}");
            included.extend(excluded.extract_if(0.., |v| {
                v.mine
                    .area_buffered()
                    .get_corner_points()
                    .iter()
                    .any(|v| processed_area.contains_point(v))
            }));
        }

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
        assert_eq!(remaining_before_len, self.mines_remaining.len());

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

        WinderNext::Continue
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
