use crate::navigator::base_source::{BaseSource, BaseSourceEighth};
use crate::navigator::circleify::draw_circle_around;
use crate::navigator::mine_executor::{
    ExecuteFlags, ExecutorResult, FailingStats, execute_route_batch_clone_prep,
};
use crate::navigator::mine_permutate::{CompletePlan, get_possible_routes_for_batch};
use crate::navigator::mine_selector::{MineSelectBatch, group_nearby_patches};
use crate::navigator::planners::PathingTunables;
use crate::navigator::planners::common_debug::{Debugger, draw_prep_mines};
use crate::surface::pixel::Pixel;
use crate::surfacev::mine::MineLocation;
use crate::surfacev::vsurface::{
    MineRef, VSurface, VSurfaceMineAsVs, VSurfaceNavAsVsMut, VSurfaceNavMut, VSurfacePatch,
    VSurfacePatchAsVs, VSurfacePatchAsVsMut, VSurfacePatchMut, VSurfacePixel, VSurfacePixelAsVs,
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
    let base_source = BaseSource::from_central_base(tunables);
    let base_source_positive = base_source.into_positive();

    find_mines(&mut surface.patches_mut(), &base_source_positive, tunables);

    Quester::init(tunables, &mut surface.nav_mut(), base_source_positive).start()
}

fn find_mines(
    surface: &mut VSurfacePatchMut,
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
    surface.set_mines(mines);
}

struct Quester<'t, 'sr, 's> {
    surface: &'sr mut VSurfaceNavMut<'s>,
    base_source: BaseSourceEighth,
    scanner: QuesterScanner,
    tunables: &'t PathingTunables,
    maybe_ban_mines: Vec<MineRef>,
}

impl<'t, 'sr, 's> Quester<'t, 'sr, 's>
where
    'sr: 's,
{
    fn init(
        tunables: &'t PathingTunables,
        surface: &'sr mut VSurfaceNavMut<'s>,
        // scanner: &'plan_mine mut QuesterScanner,
        base_source: BaseSourceEighth,
    ) -> Self {
        draw_prep_mines(&mut surface.patches_mut(), &base_source);

        let scanner = QuesterScanner::new(
            QuesterScannerBase {
                step_size: tunables.altare().step_size,
                origin: VPoint::new(0, 0),
                direction_advancing: FacDirectionQuarter::South,
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
        }
    }

    fn start(&mut self) {
        let mut limiter_counter = 0;
        let mut state = ScannerMode::Normal;

        let mut total_iterations = 0;
        loop {
            info!("iteration {}", total_iterations);
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
        let mut mines: Vec<MineRef> = match state {
            state @ ScannerMode::Normal
            //| state @ ScannerMode::Recovering(_)
            => {
                let scanned: QuesterScannerResult = self.scanner.scan(PointAt::Normal, self.surface.patches());
                match scanned {
                    QuesterScannerResult::NoneFound => {
                        self.scanner.increment(self.surface.pixels())?;
                    }
                    QuesterScannerResult::NewPatchesInScanArea { selected_mines } => {
                        trace!(
                            "scanner {} selected for state {state}",
                            selected_mines.len()
                        );
                        assert!(!selected_mines.is_empty());

                        let mandatory = selected_mines
                            .iter()
                            .filter(|v| selected_mines.contains(v) && !self.scanner.banned_mines.contains(v))
                            .take(self.tunables.altare().queue_scan)
                            .cloned()
                            .collect::<Vec<_>>();
                        if mandatory.is_empty() {
                            trace!("all patches found in scan area");
                            self.scanner.increment(self.surface.pixels())?;
                        }
                        // self.queue_redo(&mut mines);
                        return ControlFlow::Continue(mandatory);
                    }
                }
                return self.get_mines(state)
            }
            ScannerMode::Mandatory(selected_mines) => {
                assert!(!selected_mines.is_empty());
                let mut banned_mine = None;
                for (i, mine) in selected_mines.iter().enumerate() {
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
                    selected_mines.remove(i);
                }
                self.scanner.reset();

                // let mut mines = std::mem::take(selected_mines);
                // trace!("scanner {} mandatory", mines.len());
                // self.queue_redo(&mut mines);
                // mines
                std::mem::take(selected_mines)
            }

        };

        self.queue_redo(&mut mines);
        trace!("scanner and redo made {} mines", mines.len());

        let prev_len = mines.len();
        mines.dedup();
        if mines.len() != prev_len {
            panic!("dedupe detected for mines");
        }
        ControlFlow::Continue(mines)
    }

    fn execute(&mut self, state: ScannerMode, mines: Vec<MineRef>) -> ControlFlow<(), ScannerMode> {
        assert!(!mines.is_empty());
        let mines_bak = mines.clone();
        let possible_routes = self.new_plan(mines);
        if possible_routes.sequences.is_empty() {
            error!("[FATAL] no routes");
            Debugger(self.surface, "no routes")
                .starts_numbered(&self.base_source, mines_bak.len())
                .mines(mines_bak, &self.base_source);
            return ControlFlow::Break(());
        }
        info!("batch has {} sequences", possible_routes.sequences.len());

        let limit_area = possible_routes.sequences[0].routes()[0]
            .finding_limiter
            .clone();
        for mine in &mines_bak {
            assert!(
                limit_area.contains_points_all(
                    mine.get_mine(self.surface.mines())
                        .area_min()
                        .get_corner_points()
                )
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
                    let marker_mines = surface
                        .mines()
                        .get_mines_ref_for(stats.seen_mines.mines())
                        .collect_vec();
                    Debugger(surface, key)
                        .fail_mine_color_and_best_routes(stats.best_meta.unwrap())
                        .mines(marker_mines, &self.base_source)
                        .starts_numbered(&self.base_source, stats.seen_mines.len())
                        .wasteds(stats.wasteds);
                };

                let result = if self.surface.rails().get_paths().is_empty() {
                    error!("failed on first iteration, stopping");
                    apply_debug(self.surface, stats, "first-iteration");
                    ControlFlow::Break(())
                } else if stats.seen_mines.counts().all_equal()
                    && *stats.seen_mines.counts().next().unwrap() == 0
                {
                    let found: usize = stats.seen_mines.counts().sum();
                    error!("Potential deadlock, 0 mines found {found} total",);
                    apply_debug(self.surface, stats, "potential-deadlock");
                    ControlFlow::Break(())
                // } else if !stats.wasted_per_len.is_empty() {
                //     error!("why you wasting attempts?");
                //     apply_debug(self.surface, stats, "wasting-iteration");
                //     ControlFlow::Break(())
                } else {
                    match state {
                        ScannerMode::Normal => {
                            /// theory: for the least used mine, find the closest rail, undo to it, then only path to that mine
                            let lucky_mine = stats.seen_mines.least_known();

                            let nearest_mine =
                                detect_nearby_rails_as_mine_index(self.surface.rails(), lucky_mine);
                            // let total_paths = self.surface.rails().get_mine_paths().len();
                            self.base_source.undo_mine_path_until_index(
                                &mut self.surface.rails_mut(),
                                nearest_mine,
                            );
                            self.scanner.reset();

                            // assert_eq!(
                            //     i,
                            //     total_paths - nearest_path_index,
                            //     "total_paths {total_paths} nearest_path_index {nearest_path_index}"
                            // );

                            warn!(
                                "[state] execute Normal > Failure - Mandatory for {lucky_mine:?}"
                            );

                            ControlFlow::Continue(ScannerMode::Mandatory(vec![
                                self.surface.mines().get_mine_ref_for(lucky_mine),
                            ]))
                        }
                        ScannerMode::Mandatory(_) => {
                            error!("[state] execute Mandatory > Failure, breaking");
                            apply_debug(self.surface, stats, "failure-under-mandatory");
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

    fn queue_redo(&mut self, mines: &mut Vec<MineRef>) {
        let total = self.tunables.altare().queue_redo;
        for i in 0..total {
            trace!("🠋🠋🠋🠋🠋 queuing {i}/{} redo mine", total.saturating_sub(i));

            if let Some((mine, removed_points, source)) = self
                .base_source
                .undo_mine_path(&mut self.surface.rails_mut())
            {
                MineLocation::restore_area_buffered(
                    &vec![mine.location.get_mine(self.surface.mines())],
                    &mut self.surface.pixels_mut(),
                    removed_points,
                );
                mines.push(mine.location);
            } else {
                trace!("🠉🠉🠉🠉🠉🠉 queuing done");
            }
        }
    }

    fn execute_plan(&mut self, possible_routes: CompletePlan<'s>) -> PlanContinue<'s> {
        match execute_route_batch_clone_prep(
            self.tunables.mori(),
            &mut self.surface.pixels_mut(),
            possible_routes.sequences,
            &self.base_source,
            &[ExecuteFlags::ShrinkBases],
        ) {
            ExecutorResult::Success { paths, sequence: _ } => {
                let base_index_pre = self.base_source.get_i();
                let sorted_paths = self.base_source.advance_sorting(paths);
                trace!("[TMP] {base_index_pre} to {}", self.base_source.get_i());
                for path in sorted_paths {
                    self.surface.rails_mut().add_mine_path(path);
                }
                PlanContinue::Success
            }
            ExecutorResult::Failure { stats } => {
                error!(">>>>>>>> Batch fail");
                PlanContinue::Fail { stats }
            }
        }
    }

    fn new_plan(&self, mut mines: Vec<MineRef>) -> CompletePlan<'s> {
        let pre_len = mines.len();
        mines.sort();
        mines.dedup();
        assert_eq!(mines.len(), pre_len, "dedupe detected");
        let mines: Vec<(MineRef, &MineLocation)> = self
            .surface
            .mines()
            .resolve_mines_with_refs(mines)
            .collect();
        get_possible_routes_for_batch(self.surface.pixels(), MineSelectBatch { mines })
    }

    // fn scan_scanner(&self, point_at: PointAt) -> QuesterScannerResult<'s> {
    //     self.scanner.scan(point_at, self.surface.patches())
    // }
    //
    // fn mines_iter(&'s self) -> impl Iterator<Item = &'s MineLocation> + 'sr {
    //     self.patches().mines_iter()
    // }
    //
    fn patches(&self) -> VSurfacePatch<'s> {
        self.surface.patches()
    }
}

struct QuesterScannerBase {
    step_size: usize,
    origin: VPoint,
    direction_advancing: FacDirectionQuarter,
    direction_scanning: FacDirectionQuarter,
}

/// Concerned only with scanning the remaining mines
struct QuesterScanner {
    base: QuesterScannerBase,
    advance_i: usize,
    scanning_i: usize,
    reset_scanning: usize,
    banned_mines: Vec<MineRef>,
}

impl QuesterScanner {
    fn new(base: QuesterScannerBase, surface: VSurfacePatch) -> Self {
        let mut new = Self {
            base,
            advance_i: 0,
            scanning_i: 0,
            reset_scanning: 0,
            banned_mines: Vec::new(),
        };
        while !surface
            .pixels()
            .is_points_out_bounds_slice(PointAt::Normal.area_at_init(&new).get_points())
        {
            new.reset_scanning += 1;
            new.scanning_i += 1;
            assert!(new.reset_scanning < 100);
        }
        new.scanning_i = 0;
        // new.scanning_i -= 1;
        // new.reset_scanning -= 1;
        assert!(new.reset_scanning > 0);
        new
    }

    pub fn reset(&mut self) {
        self.advance_i = 0;
        self.scanning_i = 0;
    }

    fn increment(&mut self, surface: VSurfacePixel) -> ControlFlow<()> {
        trace!(
            "incrementing {} and {} reset {}",
            self.advance_i, self.scanning_i, self.reset_scanning
        );
        if self.scanning_i == self.reset_scanning {
            self.advance_i += 1;
            self.scanning_i = 0;

            let next = PointAt::Normal.area_at_init(self);
            if surface.is_points_out_bounds_slice(next.get_corner_points()) {
                trace!(
                    "incrementing out of bounds for {} and {}",
                    self.advance_i, self.scanning_i
                );
                return ControlFlow::Break(());
            }
        } else {
            self.scanning_i += 1;
        }
        ControlFlow::Continue(())
    }

    fn scan(&self, end_at: PointAt, surface: VSurfacePatch) -> QuesterScannerResult {
        // let scan_start = self.base.area_at(self.advance_i, self.scanning_i);
        let scan_area = end_at.area_at(self);

        let mut new_mines_in_scan_area: Vec<&MineLocation> = surface
            .get_mines()
            .into_iter()
            .filter(|v| scan_area.contains_point(&v.area_min().point_center()))
            .collect();
        if new_mines_in_scan_area.is_empty() {
            warn!("scan found no mines in {}", scan_area);
            return QuesterScannerResult::NoneFound;
        }
        new_mines_in_scan_area.sort_by_key(|mine| {
            let mine_pos = mine.area_min().point_center();
            // v2 bias closer to scan_direction
            let scanning =
                |point: VPoint| -> i32 { point.axis_value(self.base.direction_scanning) };
            let scanning_axis_score = scanning(mine_pos).abs_diff(scanning(self.base.origin));

            let advancing =
                |point: VPoint| -> i32 { point.axis_value(self.base.direction_advancing) };
            let advancing_axis_score =
                advancing(mine_pos).abs_diff(advancing(self.base.origin)) * 2;

            scanning_axis_score + advancing_axis_score
        });
        new_mines_in_scan_area.reverse();
        info!(
            "discovered {} mines in {scan_area}",
            new_mines_in_scan_area.len()
        );

        QuesterScannerResult::NewPatchesInScanArea {
            selected_mines: surface
                .mines()
                .get_mines_ref_for(new_mines_in_scan_area)
                .collect(),
        }
    }
}

enum QuesterScannerResult {
    NoneFound,
    NewPatchesInScanArea { selected_mines: Vec<MineRef> },
}

#[derive(Clone, Copy)]
enum PointAt {
    Normal,
    Reduced,
}

impl PointAt {
    fn area_at_init(&self, scanner: &QuesterScanner) -> VArea {
        self._area_at(scanner, true)
    }

    fn area_at(&self, scanner: &QuesterScanner) -> VArea {
        self._area_at(scanner, false)
    }

    fn _area_at(
        &self,
        QuesterScanner {
            base:
                QuesterScannerBase {
                    origin,
                    direction_advancing,
                    direction_scanning,
                    step_size,
                    ..
                },
            scanning_i,
            advance_i,
            reset_scanning,
            ..
        }: &QuesterScanner,
        is_init: bool,
    ) -> VArea {
        let scanning = if is_init {
            // during init we are still calculating
            *scanning_i
        } else {
            reset_scanning.checked_sub(*scanning_i).unwrap()
        };
        let start = origin
            .move_direction_usz(direction_advancing, step_size * advance_i)
            .move_direction_usz(direction_scanning, step_size * scanning);
        let end = match self {
            PointAt::Normal => start
                .move_direction_usz(direction_advancing, *step_size)
                .move_direction_usz(direction_scanning, *step_size),
            PointAt::Reduced => start
                .move_direction_usz(direction_advancing, step_size / 2)
                .move_direction_usz(direction_scanning, step_size / 2),
        };
        VArea::from_arbitrary_points_pair(&start, &end)
    }
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
enum PlanContinue<'plan_mine> {
    Success,
    Fail { stats: FailingStats<'plan_mine> },
    Break,
}

//

fn detect_nearby_rails_as_mine_index<'surface>(
    surface: VSurfaceRail<'surface>,
    mine_location: &MineLocation,
) -> usize {
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
