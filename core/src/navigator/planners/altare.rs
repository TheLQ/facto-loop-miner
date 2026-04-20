use crate::navigator::base_source::{BaseSource, BaseSourceEighth};
use crate::navigator::circleify::draw_circle_around;
use crate::navigator::mine_executor::{
    ExecuteFlags, ExecutorResult, FailingMeta, execute_route_batch_clone_prep,
};
use crate::navigator::mine_permutate::{CompletePlan, get_possible_routes_for_batch};
use crate::navigator::mine_selector::{MineSelectBatch, group_nearby_patches};
use crate::navigator::mori::{MoriResult, count_link_origins, mori2_start};
use crate::navigator::planners::PathingTunables;
use crate::navigator::planners::common::{Debugger, draw_prep_mines};
use crate::surface::pixel::Pixel;
use crate::surfacev::mine::MineLocation;
use crate::surfacev::vsurface::{
    VSurfaceNavMut, VSurfacePatchAsVs, VSurfacePixel, VSurfacePixelAsVs, VSurfacePixelAsVsMut,
    VSurfaceRail, VSurfaceRailAsVs, VSurfaceRailAsVsMut,
};
use facto_loop_miner_fac_engine::common::varea::VArea;
use facto_loop_miner_fac_engine::common::vpoint::VPoint;
use facto_loop_miner_fac_engine::common::vpoint_direction::{VPointDirectionQ, VSegment};
use facto_loop_miner_fac_engine::game_blocks::rail_hope::RailHopeLink;
use facto_loop_miner_fac_engine::game_blocks::rail_hope_single::SECTION_POINTS_I32;
use facto_loop_miner_fac_engine::game_entities::direction::FacDirectionQuarter;
use itertools::Itertools;
use simd_json::prelude::ArrayTrait;
use std::collections::{HashMap, HashSet};
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
    Quester::init(tunables, surface).start()
}

fn remove_bad_mines(surface: VSurfacePixel, all_mine_locations: &mut Vec<MineLocation>) {
    all_mine_locations.retain_mut(|mine| {
        mine.revalidate_endpoints_after_no_touch(surface);
        // !mine.endpoints().is_empty()
        if mine.destinations().next().is_none() {
            trace!("removing empty mine {mine:?}");
            false
        } else {
            true
        }
    });
    assert!(!all_mine_locations.is_empty());
}

struct Quester<'t, 'sr, 's> {
    surface: &'sr mut VSurfaceNavMut<'s>,
    base_source_positive: BaseSourceEighth,
    window: QuesterScanner,
    tunables: &'t PathingTunables,
}

impl<'t, 'sr, 's> Quester<'t, 'sr, 's> {
    fn init(tunables: &'t PathingTunables, surface: &'sr mut VSurfaceNavMut<'s>) -> Self {
        let base_source = BaseSource::from_central_base(tunables);
        let base_source_positive = base_source.into_positive();

        let mines_remain = group_nearby_patches(surface.patches());
        draw_prep_mines(
            &mut surface.pixels_mut(),
            &mines_remain,
            &base_source_positive,
        );

        assert!(surface.rails().get_mine_paths().is_empty());

        Quester {
            surface,
            base_source_positive,
            window: QuesterScanner::new(
                QuesterScannerBase {
                    step_size: tunables.altare().step_size,
                    origin: VPoint::new(0, 0),
                    direction_advancing: FacDirectionQuarter::North,
                    direction_scanning: FacDirectionQuarter::East,
                },
                mines_remain,
            ),
            tunables,
        }
    }

    fn start(&mut self) {
        let mut limiter_counter = 0;
        let mut state = ScannerMode::Normal;
        loop {
            let mines = match &mut state {
                ScannerMode::Normal => match self.window.scan_normal_square(self) {
                    QuesterScannerResult::AxisEnd(ScanAxis::Advance) => {
                        info!("base_source out of bounds, ending");
                        break;
                    }
                    QuesterScannerResult::AxisEnd(ScanAxis::Scanner) => {
                        self.window.increment_advance();
                        continue;
                    }
                    QuesterScannerResult::NoneFound => {
                        self.window.increment_scanner();
                        continue;
                    }
                    QuesterScannerResult::NewPatchesInScanArea { selected_mines } => {
                        trace!("scanner {} selected", selected_mines.len());
                        assert!(!selected_mines.is_empty());

                        let surface_mines: Vec<&MineLocation> = self
                            .surface
                            .rails()
                            .get_mine_paths()
                            .iter()
                            .map(|v| &v.location)
                            .collect::<Vec<_>>();

                        let mut mines = selected_mines
                            .into_iter()
                            .filter(|v| !surface_mines.contains(&v))
                            .take(self.tunables.altare().queue_scan)
                            .collect::<Vec<_>>();
                        if mines.is_empty() {
                            trace!("all patches found in scan area");
                            self.window.increment_scanner();
                            continue;
                        }

                        self.queue_redo(&mut mines);
                        trace!("scanner and redo made {} mines", mines.len());
                        mines
                    }
                },
                ScannerMode::Mandatory(selected_mines) => {
                    trace!("scanner {} mandatory", selected_mines.len());
                    self.queue_redo(selected_mines);
                    std::mem::take(selected_mines)
                }
            };

            if limiter_counter >= 99999 {
                self.debug_iteration(limiter_counter);
                break;
            }
            limiter_counter += 1;

            assert!(!mines.is_empty());
            let mines_bak = mines.clone();
            let possible_routes = self.new_plan(mines);
            if possible_routes.sequences.is_empty() {
                error!("[FATAL] no routes");
                Debugger(self.surface)
                    .starts_numbered(
                        self.base_source_positive
                            .clone()
                            .take(mines_bak.len())
                            .map(|v| *v.origin.point())
                            .collect::<Vec<_>>(),
                    )
                    .mines(&mines_bak);
                break;
            }
            info!("batch has {} sequences", possible_routes.sequences.len());

            match self.execute_plan(possible_routes) {
                ControlFlow::Break(()) => break,
                ControlFlow::Continue(PlanContinue::Success) => {
                    match state {
                        ScannerMode::Normal => {}
                        ScannerMode::Mandatory(_) => {
                            info!("[state] Clearing {state}")
                        }
                    };
                    state = ScannerMode::Normal;
                }
                ControlFlow::Continue(PlanContinue::Fail_SeenMines(meta, seen_mines)) => {
                    if seen_mines.counts().all_equal() && *seen_mines.counts().next().unwrap() == 0
                    {
                        error!(
                            "Potential deadlock, 0 mines found {} total",
                            seen_mines.len()
                        );
                        Debugger(self.surface).routes_found_notfound(meta);
                        break;
                    } else {
                        match state {
                            ScannerMode::Normal => {}
                            ScannerMode::Mandatory(_) => {
                                error!("{state} followed by {state}");
                                Debugger(self.surface).routes_found_notfound(meta);
                                break;
                            }
                        }

                        let next_mine = self.rollback_closest_rail(seen_mines);
                        state = ScannerMode::Mandatory(vec![next_mine])
                    }
                }
            }
        }
        info!("last send to oculante");
        self.surface
            .pixels()
            .paint_pixel_colored_entire()
            .save_to_oculante();
        info!("Closing altare")
    }

    fn debug_iteration(&self, limiter_counter: u32) {
        // best = 16
        // better = 28, 30, 32
        info!("limiter {limiter_counter}");
        // break;
        let start = self.base_source_positive.origin();
        let end = VPointDirectionQ(
            VPoint::new(SECTION_POINTS_I32 * 100, SECTION_POINTS_I32 * 100),
            FacDirectionQuarter::East,
        );
        let surface = self.surface.pixels();

        let fixed_radius = surface.get_radius_i32();
        let fixed_finding_limiter = VArea::from_arbitrary_points_pair(
            VPoint::new(0, -fixed_radius),
            // Must give spacing from Edge, because hope_link.area() can extend past it.
            // range checks are disabled for theoretical performance
            VPoint::new(fixed_radius, fixed_radius),
        );

        let result = mori2_start(
            self.tunables.mori(),
            surface,
            VSegment { start, end },
            &fixed_finding_limiter,
        );
        let MoriResult::FailingDebug { err } = result else {
            panic!("it worked? {end}")
        };
        surface
            .paint_pixel_graduated(count_link_origins(&err.seen))
            .save_to_oculante();
    }

    fn queue_redo(&mut self, mines: &mut Vec<MineLocation>) {
        let total = self.tunables.altare().queue_redo;
        for i in 0..total {
            trace!("🠋🠋🠋🠋🠋 queuing {i}/{} redo mine", total.saturating_sub(i));
            if let Some((mine, removed_points)) = self.surface.rails_mut().remove_mine_path_pop() {
                MineLocation::restore_area_buffered(
                    &[&mine.location],
                    &mut self.surface.pixels_mut(),
                    removed_points,
                );
                mines.push(mine.location.clone());
                let last_entry = self.base_source_positive.undo_one();
                assert_eq!(last_entry.origin, mine.segment.start);
            } else {
                trace!("🠉🠉🠉🠉🠉🠉 queuing done");
            }
        }
    }

    fn execute_plan(&mut self, possible_routes: CompletePlan) -> ControlFlow<(), PlanContinue> {
        match execute_route_batch_clone_prep(
            self.tunables.mori(),
            &mut self.surface.pixels_mut(),
            possible_routes.sequences,
            &[ExecuteFlags::ShrinkBases],
        ) {
            ExecutorResult::Success { paths, routes } => {
                let base_index_pre = self.base_source_positive.get_i();
                let sorted_paths = self.base_source_positive.advance_sorting(paths);
                trace!(
                    "[TMP] {base_index_pre} to {}",
                    self.base_source_positive.get_i()
                );
                for path in sorted_paths {
                    self.surface.rails_mut().add_mine_path(path);
                }

                self.surface
                    .pixels()
                    .paint_pixel_colored_zoomed()
                    .save_to_oculante();
                ControlFlow::Continue(PlanContinue::Success)
            }
            ExecutorResult::Failure { meta, seen_mines } => {
                if self.surface.rails().get_mine_paths().is_empty() {
                    error!("failed on first iteration, stopping");
                    Debugger(self.surface).routes_found_notfound(meta);
                    ControlFlow::Break(())
                } else {
                    error!(">>>>>>>> Batch fail");

                    self.surface
                        .pixels()
                        .paint_pixel_colored_zoomed()
                        .save_to_oculante();
                    ControlFlow::Continue(PlanContinue::Fail_SeenMines(meta, SeenMines(seen_mines)))
                }
            }
        }
    }

    /// theory: for the least used mine, find the closest rail, undo to it, then only path to that mine
    fn rollback_closest_rail(&mut self, seen_mines: SeenMines) -> MineLocation {
        let lucky_mine = seen_mines.least_known();

        let nearest_path_index = detect_nearby_rails_as_index(self.surface.rails(), &lucky_mine);
        let total_paths = self.surface.rails().get_mine_paths().len();

        let mut i = 0;
        while self
            .surface
            .rails_mut()
            .remove_mine_path_pop()
            .unwrap()
            .0
            .location
            != *lucky_mine
        {
            trace!("[rollback] pop {i}");
            i += 1;
        }
        assert_eq!(
            i,
            total_paths - nearest_path_index,
            "total_paths {total_paths} nearest_path_index {nearest_path_index}"
        );

        lucky_mine.clone()
    }

    fn new_plan(&self, mut mines: Vec<MineLocation>) -> CompletePlan {
        let pre_len = mines.len();
        mines.dedup();
        assert_eq!(mines.len(), pre_len, "dedupe detected");
        get_possible_routes_for_batch(
            self.surface.pixels(),
            MineSelectBatch {
                base_sources: self.base_source_positive.clone(),
                mines,
            },
        )
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
            .move_direction_usz(self.direction_advancing.rotate_flip(), self.step_size / 2)
            .move_direction_usz(self.direction_scanning.rotate_flip(), self.step_size / 2)
    }
}

/// Concerned only with scanning the remaining mines
struct QuesterScanner {
    base: QuesterScannerBase,
    advance_i: usize,
    scanning_i: usize,
    _raw_mines: Vec<MineLocation>,
}

impl QuesterScanner {
    fn new(base: QuesterScannerBase, _raw_mines: Vec<MineLocation>) -> Self {
        Self {
            base,
            advance_i: 0,
            scanning_i: 0,
            _raw_mines,
        }
    }

    fn mines(&self) -> &[MineLocation] {
        &self._raw_mines
    }

    fn increment_scanner(&mut self) {
        self.scanning_i += 1;
    }

    fn increment_advance(&mut self) {
        self.advance_i += 1;
        self.scanning_i = 0;
    }

    fn scan_normal_square(&self, quester: &Quester) -> QuesterScannerResult {
        self.scan(
            quester,
            self.base.point_at(self.advance_i, self.scanning_i + 1),
        )
    }

    fn scan_reduced_square(&self, quester: &Quester) -> QuesterScannerResult {
        self.scan(
            quester,
            self.base
                .point_at_last_reduced(self.advance_i, self.scanning_i + 1),
        )
    }

    fn scan(&self, quester: &Quester, scan_end: VPoint) -> QuesterScannerResult {
        if quester.surface.pixels().is_point_out_of_bounds(&scan_end) {
            return QuesterScannerResult::AxisEnd(if self.scanning_i == 0 {
                ScanAxis::Advance
            } else {
                ScanAxis::Scanner
            });
        }

        // let scan_start = self.base.point_at(self.advance_i, self.scanning_i);
        let scan_start = self.base.origin;
        let scan_area = VArea::from_arbitrary_points_pair(&scan_start, &scan_end);

        let mut new_mines_in_scan_area: Vec<MineLocation> = self
            .mines()
            .iter()
            .filter(|v| scan_area.contains_point(&v.area_min().point_center()))
            .cloned()
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

enum QuesterScannerResult {
    AxisEnd(ScanAxis),
    NoneFound,
    NewPatchesInScanArea { selected_mines: Vec<MineLocation> },
}

enum ScanAxis {
    Scanner,
    Advance,
}

//

#[derive(strum::Display)]
enum ScannerMode {
    Normal,
    Mandatory(Vec<MineLocation>),
}

//

#[allow(non_camel_case_types)]
enum PlanContinue {
    Success,
    Fail_SeenMines(FailingMeta, SeenMines),
}

//

struct SeenMines(HashMap<MineLocation, usize>);

impl SeenMines {
    fn least_known(&self) -> &MineLocation {
        self.0.iter().min_by_key(|(_, count)| *count).unwrap().0
    }

    fn counts(&self) -> std::collections::hash_map::Values<'_, MineLocation, usize> {
        self.0.values()
    }

    fn len(&self) -> usize {
        self.0.len()
    }
}

fn detect_nearby_rails_as_index(surface: VSurfaceRail, mine_location: &MineLocation) -> usize {
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
                            trace!("found rail at {distance}");
                            Some((cursor, distance))
                        }
                        Some((prev_cursor, prev_distance)) if distance < prev_distance => {
                            trace!("found rail at {distance} better than {prev_distance}");
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
