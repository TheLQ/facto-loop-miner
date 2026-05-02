use crate::navigator::base_source::{BaseSource, BaseSourceEighth};
use crate::navigator::circleify::draw_circle_around;
use crate::navigator::mine_executor::{
    ExecuteFlags, ExecutorResult, FailingStats, execute_route_batch,
};
use crate::navigator::mine_permutate::{CompletePlan, get_possible_routes_for_batch};
use crate::navigator::planners::PathingTunables;
use crate::navigator::planners::common_debug::{Debugger, draw_prep_mines};
use crate::navigator::scanners::common::MineSelectBatch;
use crate::navigator::scanners::hakka::{Hakka, HakkaBase, HakkaResult, PointAt};
use crate::navigator::scanners::patch_grouper::group_nearby_patches;
use crate::surface::pixel::Pixel;
use crate::surfacev::mine::{MineDraw, MineLocation};
use crate::surfacev::vsurface::{
    MineRef, VSurface, VSurfaceMine, VSurfaceMineAsVs, VSurfaceMineAsVsMut, VSurfaceNavAsVsMut,
    VSurfaceNavMut, VSurfacePatch, VSurfacePatchAsVs, VSurfacePixel, VSurfacePixelAsVs,
    VSurfacePixelAsVsMut, VSurfaceRail, VSurfaceRailAsVs, VSurfaceRailAsVsMut,
};
use facto_loop_miner_fac_engine::common::varea::VArea;
use facto_loop_miner_fac_engine::common::vpoint::VPoint;
use facto_loop_miner_fac_engine::game_entities::direction::FacDirectionQuarter;
use itertools::Itertools;
use simd_json::prelude::ArrayTrait;
use std::collections::HashSet;
use std::ops::ControlFlow;
use tracing::{error, info, trace, warn};

/// Planner v3 "Regis Altare 🎇"
///
/// Because v0 Mori and v1 Ruze Planner get deadlocked
///
/// # Operation
///
/// Rolling re-optimizing window at frontier.
///
/// On failure to batch, for the least found mine (the lucky mine) find the closest rail.
/// It can probably use it. Rollback all paths up to it.
/// Go from that old position to the lucky mine
/// Re-pathfind from there
pub fn start_altare_planner(
    tunables: &PathingTunables,
    // surface: &mut VSurfaceNavMut
    surface: &mut VSurface,
) {
    let base_source_gen = BaseSource::from_central_base(tunables);

    surface.nav_mut_fn(|s| Quester::init(tunables, s, base_source_gen, false).start())
}

fn find_mines(
    surface: &mut VSurfaceNavMut,
    base_source: &BaseSourceEighth,
    tunables: &PathingTunables,
) {
    let mines = group_nearby_patches(surface.patches())
        .into_iter()
        .filter_map(|patch_refs| {
            MineLocation::from_patch_indexes(
                surface.patches(),
                patch_refs,
                base_source,
                tunables.path_common(),
            )
        })
        // to make sane lifetimes, immediately only deal with references
        .collect();
    surface.mines_mut_old_fn(|s| s.set_mines(mines));
}

struct Quester<'t, 's> {
    surface: VSurfaceNavMut<'s>,
    base_source: BaseSourceEighth,
    scanner: Hakka,
    tunables: &'t PathingTunables,
    maybe_ban_mines: Vec<MineRef>,
    fixed_finding_limiter: VArea,
}

impl<'t, 's> Quester<'t, 's> {
    fn init(
        tunables: &'t PathingTunables,
        mut surface: VSurfaceNavMut<'s>,
        base_source_gen: BaseSource,
        is_base_positive: bool,
    ) -> Self {
        let base_source = if is_base_positive {
            base_source_gen.into_positive()
        } else {
            base_source_gen.into_negative()
        };

        find_mines(&mut surface, &base_source, tunables);
        surface.mines_mut_fn(|s| {
            draw_prep_mines(s, &base_source);
        });

        // Limit pathing to the entire right half of the map
        let fixed_radius = surface.pixels().get_radius_i32();
        let fixed_finding_limiter = VArea::from_arbitrary_points_pair(
            base_source.fixed_limiting_start(),
            VPoint::new(
                fixed_radius,
                if is_base_positive {
                    fixed_radius
                } else {
                    -fixed_radius
                },
            ),
        );

        let scanner = Hakka::new(
            HakkaBase {
                step_size: tunables.altare().step_size,
                origin: VPoint::new(0, 0),
                direction_advancing: if is_base_positive {
                    FacDirectionQuarter::North
                } else {
                    FacDirectionQuarter::South
                },
                direction_scanning: FacDirectionQuarter::East,
            },
            surface.patches(),
        );

        assert!(surface.rails().get_paths().is_empty());

        Quester {
            surface,
            base_source,
            scanner,
            tunables,
            maybe_ban_mines: Vec::new(),
            fixed_finding_limiter,
        }
    }

    fn start(&mut self) {
        let mut limiter_counter = 0;
        let mut state = ScannerMode::Normal;

        let mut total_iterations = 0;
        loop {
            info!("🠋🠋🠋🠋🠋🠋 iteration {}", total_iterations);
            if total_iterations == 500 {
                info!("enough");
                break;
            } else {
                total_iterations += 1;
            }

            let mines: Vec<MineRef> = match self.get_mines(&mut state) {
                ControlFlow::Break(()) => break,
                ControlFlow::Continue(v) => v,
            };
            assert!(mines.len() > 1);

            if limiter_counter >= 99999 {
                self.debug_iteration(limiter_counter);
                break;
            }
            limiter_counter += 1;

            state = match self.execute(state, mines) {
                ControlFlow::Break(()) => break,
                ControlFlow::Continue(v) => v,
            };
        }
        info!("last send to oculante");
        self.common_send_to_oculante();
        info!("Closing altare")
    }

    fn get_mines(&mut self, state: &mut ScannerMode) -> ControlFlow<(), Vec<MineRef>> {
        let mut selected_mines: Vec<MineRef> = Vec::new();
        self.queue_redo(&mut selected_mines);

        match state {
            state @ ScannerMode::Normal
            //| state @ ScannerMode::Recovering(_)
            => {
                loop {
                    let scanned: HakkaResult = self.scanner.scan(PointAt::Normal, self.surface.mines());
                    match scanned {
                        HakkaResult::NoneFound => {
                            self.scanner.increment(self.surface.pixels(), "none-found")?;
                        }
                        HakkaResult::NewPatchesInScanArea { scanned_mines, scan_area } => {
                            let scanned_len = scanned_mines.len();
                            let needed_size = self.tunables.altare().queue_scan + self.tunables.altare().queue_redo;
                            assert!(!scanned_mines.is_empty());

                            let new = scanned_mines
                                .iter()
                                .filter(|v| !selected_mines.contains(v) && !self.scanner.banned_mines.contains(v) && !self.surface.rails().get_paths_mine_refs().contains(*v))
                                .take(needed_size - selected_mines.len())
                                .cloned()
                                .collect_vec();
                            if new.is_empty() {
                                self.scanner.increment(self.surface.pixels(), format!("scan filtered {scanned_len} mines in {scan_area}"))?;
                            } else {
                                selected_mines.extend(new);
                                trace!("[scanner] scan {scanned_len} total {} needed {needed_size} for state {state}", selected_mines.len());
                                if selected_mines.len() >= needed_size {
                                    break;
                                }
                            }
                        }
                    }
                }
            }
            ScannerMode::Mandatory(mandatory_mines) => {
                assert!(!mandatory_mines.is_empty());
                mandatory_mines.retain(|mine| {
                    !selected_mines.contains(mine)
                });

                let mut banned_mine = None;
                for (i, mine) in mandatory_mines.iter().enumerate() {
                    if self.maybe_ban_mines.contains(mine) {
                        // was previously mandatory
                        warn!("Banning mine {mine:?}");
                        self.scanner.banned_mines.push(*mine);
                        self.maybe_ban_mines.clear();
                        banned_mine = Some(i);
                        break;
                    } else {
                        self.maybe_ban_mines.push(*mine);
                    }
                }
                if let Some(i) = banned_mine {
                    mandatory_mines.remove(i);
                }
                self.scanner.reset();

                // let mut mines = std::mem::take(selected_mines);
                // trace!("scanner {} mandatory", mines.len());
                // self.queue_redo(&mut mines);
                // mines
                let mines = std::mem::take(mandatory_mines);
                trace!("[scanner] mandatory made {} mines", mines.len());
                selected_mines.extend(mines);
            }
        };

        let pre_len = selected_mines.len();
        selected_mines.sort();
        selected_mines.dedup();
        assert_eq!(selected_mines.len(), pre_len, "dedupe detected for mines");

        assert!(pre_len > 1, "actual mines_len {pre_len}");
        ControlFlow::Continue(selected_mines)
    }

    fn execute(&mut self, state: ScannerMode, mines: Vec<MineRef>) -> ControlFlow<(), ScannerMode> {
        assert!(!mines.is_empty());
        let mines_bak = mines.clone();
        let possible_routes = self.new_plan(mines);
        if possible_routes.sequences.is_empty() {
            error!("[FATAL] no routes");
            Debugger(&mut self.surface, "no routes")
                .starts_numbered(&self.base_source, mines_bak.len())
                .mines(mines_bak, &self.base_source);
            return ControlFlow::Break(());
        }
        // info!("batch has {} sequences", possible_routes.sequences.len());

        let limit_area = possible_routes.sequences[0].routes()[0]
            .finding_limiter
            .clone();
        for mine in &mines_bak {
            let mine_area = mine.resolve_mine_surface(self.surface.mines()).area_min();
            assert!(
                limit_area.contains_points_all(mine_area.get_corner_points(),),
                "limit_area {limit_area} for {mine:?} at {mine_area}"
            )
        }

        match self.execute_plan(possible_routes) {
            PlanContinue::Success => {
                let new_state = match state {
                    ScannerMode::Normal => {
                        info!("[state] execute Normal > Success");
                        ScannerMode::Normal
                    }
                    ScannerMode::Mandatory(prev_mandatory) => {
                        info!(
                            "[state] execute Mandatory ({}) > Success",
                            prev_mandatory.len()
                        );
                        for mine in prev_mandatory {
                            assert!(
                                self.surface
                                    .rails()
                                    .get_paths_mine_refs()
                                    .any(|surface_mine| surface_mine == mine)
                            )
                        }
                        ScannerMode::Normal
                    } // ScannerMode::Recovering(recovering) => {
                      //     if recovering.iter().any(|recover| {
                      //         self.surface
                      //             .rails()
                      //             .get_mine_paths()
                      //             .iter()
                      //             .find(|path| &path.location == *recover)
                      //             .is_some()
                      //     }) {
                      //         info!(
                      //             "[state] execute Recovering > Success - {} Recovered",
                      //             recovering.len()
                      //         );
                      //         ScannerMode::Normal
                      //     } else {
                      //         info!(
                      //             "[state] executing Recovering > Success - Waiting for {} recovering",
                      //             recovering.len()
                      //         );
                      //         ScannerMode::Recovering(recovering)
                      //     }
                      // }
                };
                self.common_send_to_oculante();
                ControlFlow::Continue(new_state)
            }
            PlanContinue::Break => ControlFlow::Break(()),
            PlanContinue::Fail { stats } => {
                error!("{stats}");

                let apply_debug = |surface: &mut VSurfaceNavMut, stats: FailingStats, key| {
                    Debugger(surface, key)
                        .fail_mine_color_and_best_routes(stats.best_meta.unwrap())
                        .mines(stats.seen_mines.mines(), &self.base_source)
                        .starts_numbered(&self.base_source, stats.seen_mines.len())
                        .wasteds(stats.wasteds);
                };

                let result = if self.surface.rails().get_paths().is_empty() {
                    error!("failed on first iteration, stopping");
                    apply_debug(&mut self.surface, stats, "first-iteration");
                    ControlFlow::Break(())
                } else if {
                    if let Some(first) = stats.seen_mines.counts().next() {
                        first == 0
                    } else {
                        true
                    }
                } && stats.seen_mines.counts().all_equal()
                {
                    error!("Potential deadlock, 0 mines found",);
                    apply_debug(&mut self.surface, stats, "potential-deadlock");
                    ControlFlow::Break(())
                // } else if !stats.wasted_per_len.is_empty() {
                //     error!("why you wasting attempts?");
                //     apply_debug(self.surface, stats, "wasting-iteration");
                //     ControlFlow::Break(())
                } else {
                    match state {
                        ScannerMode::Normal => {
                            // theory: for the least used mine, find the closest rail, undo to it, then only path to that mine
                            let lucky_mine_ref = stats.seen_mines.least_known().unwrap();

                            let nearest_mine = detect_nearby_rails_as_mine_index(
                                self.surface.rails(),
                                lucky_mine_ref.resolve_mine_surface(self.surface.mines()),
                            );
                            // let total_paths = self.surface.rails().get_mine_paths().len();
                            self.surface.rails_mut_old_fn(|s| {
                                self.base_source.undo_mine_path_until_index(s, nearest_mine)
                            });
                            self.scanner.reset();

                            // assert_eq!(
                            //     i,
                            //     total_paths - nearest_path_index,
                            //     "total_paths {total_paths} nearest_path_index {nearest_path_index}"
                            // );

                            warn!(
                                "[state] execute Normal > Failure - Mandatory for {lucky_mine_ref:?}"
                            );

                            ControlFlow::Continue(ScannerMode::Mandatory(vec![lucky_mine_ref]))
                        }
                        ScannerMode::Mandatory(_) => {
                            error!("[state] execute Mandatory > Failure, breaking");
                            apply_debug(&mut self.surface, stats, "failure-under-mandatory");
                            ControlFlow::Break(())
                        } // ScannerMode::Recovering(_) => {
                          //     error!("[state] execute Recovering > Failure, breaking");
                          //     apply_debug(self.surface, stats, "failure-under-recovery");
                          //     ControlFlow::Break(())
                          // }
                    }
                };
                self.common_send_to_oculante();
                result
            }
        }
    }

    fn common_send_to_oculante(&self) {
        self.surface
            .pixels()
            .paint_pixel_colored_entire()
            .save_to_oculante()
    }

    fn debug_iteration(&self, _limiter_counter: u32) {
        // // best = 16
        // // better = 28, 30, 32
        // info!("limiter {limiter_counter}");
        // // break;
        // let start = self.base_source_positive.origin();
        // let end = VPointDirectionQ(
        //     VPoint::new(SECTION_POINTS_I32 * 100, SECTION_POINTS_I32 * 100),
        //     FacDirectionQuarter::East,
        // );
        // let surface = self.surface.pixels();
        //
        // let fixed_radius = surface.get_radius_i32();
        // let fixed_finding_limiter = VArea::from_arbitrary_points_pair(
        //     VPoint::new(0, -fixed_radius),
        //     // Must give spacing from Edge, because hope_link.area() can extend past it.
        //     // range checks are disabled for theoretical performance
        //     VPoint::new(fixed_radius, fixed_radius),
        // );
        //
        // let result = mori2_start(
        //     self.tunables.mori(),
        //     surface,
        //     VSegment { start, end },
        //     &fixed_finding_limiter,
        // );
        // let MoriResult::FailingDebug { cause } = result else {
        //     panic!("it worked? {end}")
        // };
        // surface
        //     .paint_pixel_graduated(count_link_origins(&err.seen))
        //     .save_to_oculante();
    }

    fn undo_one(&mut self, message: impl std::fmt::Display) -> Option<MineRef> {
        if let Some((path, _removed_points, _source)) = self
            .surface
            .rails_mut_old_fn(|s| self.base_source.undo_mine_path(s, message))
        {
            self.surface
                .mines_mut_ref()
                .draw_mine(path.destination.mine_ref(), MineDraw::ChangeBuffered);

            Some(path.destination.mine_ref())
        } else {
            None
        }
    }

    fn queue_redo(&mut self, mines: &mut Vec<MineRef>) {
        let total = self.tunables.altare().queue_redo;
        for i in 0..total {
            if let Some(mine) =
                self.undo_one(format!("queuing {i}/{} redo mine", total.saturating_sub(i)))
            {
                mines.push(mine);
            } else {
                trace!("queue empty");
                break;
            }
        }
        // trace!("🠉🠉🠉🠉🠉🠉 queuing done");
    }

    fn execute_plan(&mut self, possible_routes: CompletePlan) -> PlanContinue {
        // execute_route_batch_clone_prep does this but requires mut
        self.surface.pixels_mut_ref().load_clone_prep().unwrap();

        let surface = self.surface.mines();
        let mine_resolver = surface
            .resolve_mines_with_refs(
                possible_routes.sequences[0]
                    .routes()
                    .iter()
                    .map(|v| v.destination.mine_ref()),
            )
            .collect_vec();

        match execute_route_batch(
            self.tunables.mori(),
            self.surface.pixels(),
            possible_routes.sequences,
            &self.base_source,
            mine_resolver,
            &[ExecuteFlags::ShrinkBases],
        ) {
            ExecutorResult::Success { paths, sequence: _ } => {
                let sorted_paths = self.base_source.advance_sorting(paths);
                self.surface.rails_mut_fn(|mut s| {
                    for path in sorted_paths {
                        s.add_mine_path(path, "best-batch-sequence");
                    }
                });

                PlanContinue::Success
            }
            ExecutorResult::Failure { stats } => {
                error!(">>>>>>>> Batch fail");
                PlanContinue::Fail { stats }
            }
        }
    }

    fn new_plan(&self, mines: Vec<MineRef>) -> CompletePlan {
        get_possible_routes_for_batch(
            self.surface.mines(),
            MineSelectBatch { mines },
            &self.fixed_finding_limiter,
        )
    }

    // fn scan_scanner(&self, point_at: PointAt) -> QuesterScannerResult<'s> {
    //     self.scanner.scan(point_at, self.surface.patches())
    // }
    //
    // fn mines_iter(&'s self) -> impl Iterator<Item = &'s MineLocation> + 'sr {
    //     self.patches().mines_iter()
    // }
    //
}

//

#[derive(strum::Display)]
enum ScannerMode {
    Normal,
    Mandatory(Vec<MineRef>),
    // Recovering(Vec<&'plan_mine MineLocation>),
}

//

#[allow(non_camel_case_types)]
enum PlanContinue {
    Success,
    Fail { stats: FailingStats },
    Break,
}

//

fn detect_nearby_rails_as_mine_index(surface: VSurfaceRail, mine_location: &MineLocation) -> usize {
    let origin = mine_location
        .area_min()
        .point_center()
        .move_round_even_down();
    origin.assert_even_position();

    let mut closest_rail = None;
    let mut seen_points = HashSet::new();
    for depth in 2.. {
        let mut stop_after = false;
        trace!("circling origin {origin} depth {depth}");
        for cursor in draw_circle_around(&origin, depth * 200) {
            if !cursor.is_even() || surface.pixels().is_point_out_of_bounds(&cursor) {
                continue;
            }

            if seen_points.contains(&cursor) {
                continue;
            }
            seen_points.insert(cursor);

            let distance = origin.distance_bird(&cursor).abs();
            match surface.pixels().get_pixel(cursor) {
                Pixel::Empty | Pixel::MineNoTouch | Pixel::Highlighter => {
                    // the vast expanse...
                }
                Pixel::Rail => {
                    closest_rail = match closest_rail {
                        None => {
                            // trace!("found rail at {distance}");
                            Some((cursor, distance))
                        }
                        Some((prev_cursor, prev_distance)) if distance < prev_distance => {
                            // trace!("found rail at {distance} better than {prev_distance}");
                            Some((cursor, distance))
                        }
                        Some(good) => Some(good),
                    };
                    stop_after = true;
                }
                pixel if Pixel::is_resource(&pixel) => {
                    // ignore resources
                }
                pixel => {
                    // resource buffer area probably
                    trace!("hit limit at depth {depth} at {pixel:?}");
                    stop_after = true;
                }
            }
        }
        if stop_after {
            break;
        }
    }
    let (closest_rail, _) = closest_rail.unwrap();
    trace!("closest rail at {closest_rail}");

    surface
        .get_paths()
        .iter()
        .position(|p| {
            p.links
                .iter()
                .any(|link| link.link_area_slow().contains(&closest_rail))
        })
        .unwrap_or_else(|| panic!("No rail found at {closest_rail}"))
}
