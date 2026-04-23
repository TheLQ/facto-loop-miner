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
use crate::surfacev::mine::{MineLocation, MinePath};
use crate::surfacev::vsurface::{
    VSurfaceNavMut, VSurfacePatchAsVs, VSurfacePixel, VSurfacePixelAsVs, VSurfacePixelAsVsMut,
    VSurfaceRail, VSurfaceRailAsVs, VSurfaceRailAsVsMut,
};
use facto_loop_miner_fac_engine::common::varea::VArea;
use facto_loop_miner_fac_engine::common::vpoint::VPoint;
use facto_loop_miner_fac_engine::game_blocks::rail_hope::RailHopeLink;
use facto_loop_miner_fac_engine::game_entities::direction::FacDirectionQuarter;
use itertools::Itertools;
use simd_json::prelude::ArrayTrait;
use std::collections::HashSet;
use std::ops::ControlFlow;
use tracing::{error, info, trace, warn};

/// Planner v3 "Regis Altare 🎇"
///
/// Pathfinding with medium-difficulty backtracking.
/// because v0 Mori and v1 Ruze Planner get deadlocked
///
/// # Operation
///
/// Rolling re-optimizing window at frontier.
///
/// On failure to batch, for the least found mine (the lucky mine) find the closest rail.
/// It can probably use it. Rollback all paths up to it.
/// Go from that old position to the lucky mine
/// Re-pathfind from there
pub fn start_altare_planner(tunables: &PathingTunables, surface: &mut VSurfaceNavMut) {
    let mut mines = Vec::new();
    Quester::init(tunables, surface, &mut mines).start()
}

struct Quester<'t, 'sr, 's, 'plan_mine> {
    surface: &'sr mut VSurfaceNavMut<'s>,
    base_source_positive: BaseSourceEighth,
    scanner: QuesterScanner<'plan_mine>,
    tunables: &'t PathingTunables,
}

impl<'t, 'sr, 's, 'plan_mine> Quester<'t, 'sr, 's, 'plan_mine> {
    fn init(
        tunables: &'t PathingTunables,
        surface: &'sr mut VSurfaceNavMut<'s>,
        // scanner: &'plan_mine mut QuesterScanner,
        owned_mines: &'plan_mine mut Vec<MineLocation>,
    ) -> Self {
        let base_source = BaseSource::from_central_base(tunables);
        let base_source_positive = base_source.into_positive();

        group_nearby_patches(surface.patches())
            .into_iter()
            .filter_map(|patch_refs| {
                MineLocation::from_patch_indexes(
                    surface.patches(),
                    patch_refs,
                    &base_source_positive,
                    tunables.path_common(),
                )
            })
            // to make sane lifetimes, immediately only deal with references
            .collect_into(owned_mines);
        let mines = owned_mines.iter().collect();

        let scanner = QuesterScanner::new(
            QuesterScannerBase {
                step_size: tunables.altare().step_size,
                origin: VPoint::new(0, 0),
                direction_advancing: FacDirectionQuarter::South,
                direction_scanning: FacDirectionQuarter::East,
            },
            mines,
        );

        assert!(surface.rails().get_mine_paths().is_empty());

        draw_prep_mines(
            &mut surface.pixels_mut(),
            scanner.mines(),
            &base_source_positive,
        );

        Quester {
            surface,
            base_source_positive,
            scanner,
            tunables,
        }
    }

    fn start(&mut self) {
        let mut limiter_counter = 0;
        let mut state = ScannerMode::Normal;

        loop {
            let mines: Vec<&MineLocation> = match self.get_mines(&mut state) {
                ControlFlow::Break(()) => break,
                ControlFlow::Continue(None) => continue,
                ControlFlow::Continue(Some(v)) => v,
            };

            if limiter_counter >= 99999 {
                self.debug_iteration(limiter_counter);
                break;
            }
            limiter_counter += 1;

            match self.execute(&mut state, mines.as_slice()) {
                ControlFlow::Break(()) => break,
                ControlFlow::Continue(()) => {}
            }
        }
        info!("last send to oculante");
        self.common_send_to_oculante();
        info!("Closing altare")
    }

    fn get_mines(
        &mut self,
        state: &mut ScannerMode<'plan_mine>,
    ) -> ControlFlow<(), Option<Vec<&'plan_mine MineLocation>>> {
        let mut mines: Vec<&'plan_mine MineLocation> = match state {
            ScannerMode::Normal => match self.scanner.scan_normal_square(self.surface.pixels()) {
                QuesterScannerResult::AxisEnd(ScanAxis::Advance) => {
                    info!("base_source out of bounds, ending");
                    return ControlFlow::Break(());
                }
                QuesterScannerResult::AxisEnd(ScanAxis::Scanner) => {
                    self.scanner.increment_advance();
                    return ControlFlow::Continue(None);
                }
                QuesterScannerResult::NoneFound => {
                    self.scanner.increment_scanner();
                    return ControlFlow::Continue(None);
                }
                QuesterScannerResult::NewPatchesInScanArea { selected_mines } => {
                    trace!("scanner {} selected", selected_mines.len());
                    assert!(!selected_mines.is_empty());

                    let found_mines: Vec<&'plan_mine MineLocation> = self
                        .surface
                        .rails()
                        .get_mine_paths()
                        .iter()
                        .map(|v| {
                            let surface_location = &v.location;
                            // need location owned by planner
                            self.scanner
                                .mines()
                                .find_or_first(|v| *v == surface_location)
                                .unwrap()
                        })
                        .collect::<Vec<_>>();

                    let mut mines = selected_mines
                        .into_iter()
                        .filter(|v| !found_mines.contains(&v))
                        .take(self.tunables.altare().queue_scan)
                        .collect::<Vec<_>>();
                    if mines.is_empty() {
                        trace!("all patches found in scan area");
                        self.scanner.increment_scanner();
                        return ControlFlow::Continue(None);
                    }

                    self.queue_redo(&mut mines);
                    trace!("scanner and redo made {} mines", mines.len());

                    mines
                }
            },
            ScannerMode::Mandatory(selected_mines) => {
                let mut mines = std::mem::take(selected_mines);
                trace!("scanner {} mandatory", mines.len());
                self.queue_redo(&mut mines);
                mines
            }
        };

        let prev_len = mines.len();
        mines.dedup();
        if mines.len() != prev_len {
            panic!("dedupe detected for mines");
        }
        ControlFlow::Continue(Some(mines))
    }

    fn execute(
        &mut self,
        state: &mut ScannerMode<'plan_mine>,
        mines: &[&'plan_mine MineLocation],
    ) -> ControlFlow<()> {
        assert!(!mines.is_empty());
        let mines_bak: Vec<&'plan_mine MineLocation> = mines.to_vec();
        let possible_routes = self.new_plan(mines.to_vec());
        if possible_routes.sequences.is_empty() {
            error!("[FATAL] no routes");
            Debugger(self.surface, "no routes")
                .starts_numbered(&self.base_source_positive, mines_bak.len())
                .mines(mines_bak.iter().map(|v| *v), &self.base_source_positive);
            return ControlFlow::Break(());
        }
        info!("batch has {} sequences", possible_routes.sequences.len());

        match self.execute_plan(possible_routes) {
            PlanContinue::Success => {
                match state {
                    ScannerMode::Normal => {}
                    ScannerMode::Mandatory(_) => {
                        info!("[state] Clearing {state}")
                    }
                };
                *state = ScannerMode::Normal;

                self.common_send_to_oculante();
            }
            PlanContinue::Fail { stats } => {
                error!("{stats}");

                let apply_debug = |surface, stats: FailingStats, key| {
                    Debugger(surface, key)
                        .fail_mine_color_and_best_routes(stats.best_meta.unwrap())
                        .mines(stats.seen_mines.mines(), &self.base_source_positive)
                        .starts_numbered(&self.base_source_positive, stats.seen_mines.len())
                        .wasteds(stats.wasteds);
                };

                let is_break;
                // if !stats.wasted_per_len.is_empty() {
                //     error!("why you wasting attempts?");
                //     apply_debug(self.surface, stats, "wasting-iteration");
                //     is_break = true;
                // } else
                if self.surface.rails().get_mine_paths().is_empty() {
                    error!("failed on first iteration, stopping");
                    apply_debug(self.surface, stats, "first-iteration");
                    is_break = true;
                } else if stats.seen_mines.counts().all_equal()
                    && *stats.seen_mines.counts().next().unwrap() == 0
                {
                    let found: usize = stats.seen_mines.counts().sum();
                    error!("Potential deadlock, 0 mines found {found} total",);
                    apply_debug(self.surface, stats, "potential-deadlock");
                    is_break = true;
                } else {
                    match state {
                        ScannerMode::Normal => {
                            /// theory: for the least used mine, find the closest rail, undo to it, then only path to that mine
                            let lucky_mine = stats.seen_mines.least_known();

                            let nearest_mine =
                                detect_nearby_rails_as_mine_index(self.surface.rails(), lucky_mine);
                            // let total_paths = self.surface.rails().get_mine_paths().len();
                            self.base_source_positive.undo_mine_path_until_index(
                                &mut self.surface.rails_mut(),
                                nearest_mine,
                            );

                            // assert_eq!(
                            //     i,
                            //     total_paths - nearest_path_index,
                            //     "total_paths {total_paths} nearest_path_index {nearest_path_index}"
                            // );

                            *state = ScannerMode::Mandatory(vec![lucky_mine]);
                            is_break = false;
                        }
                        ScannerMode::Mandatory(_) => {
                            error!("{state} followed by {state}");
                            apply_debug(self.surface, stats, "Mandatory-dupe");
                            is_break = true;
                        }
                    }
                }

                self.common_send_to_oculante();
                if is_break {
                    trace!("breaking on fail");
                    return ControlFlow::Break(());
                }
            }
            PlanContinue::Break => return ControlFlow::Break(()),
        }
        ControlFlow::Continue(())
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

    fn queue_redo(&mut self, mines: &mut Vec<&'plan_mine MineLocation>) {
        let total = self.tunables.altare().queue_redo;
        for i in 0..total {
            trace!("🠋🠋🠋🠋🠋 queuing {i}/{} redo mine", total.saturating_sub(i));

            if let Some((mine, removed_points, source)) = self
                .base_source_positive
                .undo_mine_path(&mut self.surface.rails_mut())
            {
                MineLocation::restore_area_buffered(
                    &[&mine.location],
                    &mut self.surface.pixels_mut(),
                    removed_points,
                );
                mines.push(self.scanner.mines().find(|v| **v == mine.location).unwrap());
            } else {
                trace!("🠉🠉🠉🠉🠉🠉 queuing done");
            }
        }
    }

    fn execute_plan(
        &mut self,
        possible_routes: CompletePlan<'plan_mine>,
    ) -> PlanContinue<'plan_mine> {
        match execute_route_batch_clone_prep(
            self.tunables.mori(),
            &mut self.surface.pixels_mut(),
            possible_routes.sequences,
            &self.base_source_positive,
            &[ExecuteFlags::ShrinkBases],
        ) {
            ExecutorResult::Success { paths, sequence: _ } => {
                let base_index_pre = self.base_source_positive.get_i();
                let sorted_paths = self.base_source_positive.advance_sorting(paths);
                trace!(
                    "[TMP] {base_index_pre} to {}",
                    self.base_source_positive.get_i()
                );
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

    fn new_plan(&self, mut mines: Vec<&'plan_mine MineLocation>) -> CompletePlan<'plan_mine> {
        let pre_len = mines.len();
        mines.dedup();
        assert_eq!(mines.len(), pre_len, "dedupe detected");
        get_possible_routes_for_batch(self.surface.pixels(), MineSelectBatch { mines })
    }
}

struct QuesterScannerBase {
    step_size: usize,
    origin: VPoint,
    direction_advancing: FacDirectionQuarter,
    direction_scanning: FacDirectionQuarter,
}

impl QuesterScannerBase {
    fn point_at(&self, advanced: usize, scanning: usize) -> VPoint {
        self.origin
            .move_direction_usz(self.direction_advancing, self.step_size * advanced)
            .move_direction_usz(self.direction_scanning, self.step_size * scanning)
    }

    fn point_at_last_reduced(&self, advanced: usize, scanning: usize) -> VPoint {
        self.point_at(advanced, scanning)
            .move_direction_usz(
                self.direction_advancing.rotate_flip(),
                (self.step_size * advanced) / 2,
            )
            .move_direction_usz(
                self.direction_scanning.rotate_flip(),
                (self.step_size * scanning) / 2,
            )
    }
}

/// Concerned only with scanning the remaining mines
struct QuesterScanner<'plan_mine> {
    base: QuesterScannerBase,
    advance_i: usize,
    scanning_i: usize,
    _raw_mines: Vec<&'plan_mine MineLocation>,
}

impl<'plan_mine> QuesterScanner<'plan_mine> {
    fn new(base: QuesterScannerBase, mines: Vec<&'plan_mine MineLocation>) -> Self {
        Self {
            base,
            advance_i: 0,
            scanning_i: 0,
            _raw_mines: mines,
        }
    }

    fn mines(&self) -> impl Iterator<Item = &'plan_mine MineLocation> {
        self._raw_mines.iter().map(|v| *v)
    }

    fn increment_scanner(&mut self) {
        self.scanning_i += 1;
    }

    fn increment_advance(&mut self) {
        self.advance_i += 1;
        self.scanning_i = 0;
    }

    fn scan_normal_square(&self, surface: VSurfacePixel) -> QuesterScannerResult<'plan_mine> {
        self.scan(
            surface,
            self.base.point_at(self.advance_i, self.scanning_i + 1),
        )
    }

    fn scan_reduced_square(&self, surface: VSurfacePixel) -> QuesterScannerResult<'plan_mine> {
        self.scan(
            surface,
            self.base
                .point_at_last_reduced(self.advance_i, self.scanning_i + 1),
        )
    }

    fn scan(&self, surface: VSurfacePixel, scan_end: VPoint) -> QuesterScannerResult<'plan_mine> {
        if surface.is_point_out_of_bounds(&scan_end) {
            return QuesterScannerResult::AxisEnd(if self.scanning_i == 0 {
                ScanAxis::Advance
            } else {
                ScanAxis::Scanner
            });
        }

        // let scan_start = self.base.point_at(self.advance_i, self.scanning_i);
        let scan_start = self.base.origin;
        let scan_area = VArea::from_arbitrary_points_pair(&scan_start, &scan_end);

        let mut new_mines_in_scan_area: Vec<&MineLocation> = self
            .mines()
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
            let scanning_axis_score = scanning(mine_pos).abs_diff(scanning(self.base.origin)) / 2;

            let advancing =
                |point: VPoint| -> i32 { point.axis_value(self.base.direction_advancing) };
            let advancing_axis_score = advancing(mine_pos).abs_diff(advancing(self.base.origin));

            scanning_axis_score + advancing_axis_score
        });
        info!(
            "discovered {} mines in {scan_area}",
            new_mines_in_scan_area.len()
        );

        QuesterScannerResult::NewPatchesInScanArea {
            selected_mines: new_mines_in_scan_area,
        }
    }
}

enum QuesterScannerResult<'m> {
    AxisEnd(ScanAxis),
    NoneFound,
    NewPatchesInScanArea {
        selected_mines: Vec<&'m MineLocation>,
    },
}

enum ScanAxis {
    Scanner,
    Advance,
}

//

#[derive(strum::Display)]
enum ScannerMode<'plan_mine> {
    Normal,
    Mandatory(Vec<&'plan_mine MineLocation>),
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
        .get_mine_paths()
        .iter()
        .position(|p| {
            p.links
                .iter()
                .any(|link| link.area_vec().contains(&closest_rail))
        })
        .unwrap_or_else(|| panic!("No rail found at {closest_rail}"))
}
