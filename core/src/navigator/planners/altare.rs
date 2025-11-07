use crate::navigator::base_source::{BaseSource, BaseSourceEighth};
use crate::navigator::circleify::draw_circle_around;
use crate::navigator::mine_executor::{
    ExecuteFlags, ExecutorResult, execute_route_batch, execute_route_batch_clone_prep,
};
use crate::navigator::mine_permutate::{CompletePlan, get_possible_routes_for_batch};
use crate::navigator::mine_selector::{
    MineSelectBatch, PERPENDICULAR_SCAN_WIDTH, group_nearby_patches,
};
use crate::navigator::mori::{MoriResult, count_link_origins, mori2_start};
use crate::navigator::planners::PathingTunables;
use crate::navigator::planners::common::{
    debug_draw_failing_mines, debug_draw_mine_index_labels, debug_draw_mine_links, debug_failing,
    draw_prep_mines,
};
use crate::state::machine::StepParams;
use crate::state::tuneables::{MoriTunables, Tunables};
use crate::surface::pixel::Pixel;
use crate::surfacev::mine::MineLocation;
use std::cell::RefCell;

use crate::surfacev::vsurface::{
    VSurface, VSurfaceNavMut, VSurfacePatchAsVs, VSurfacePixel, VSurfacePixelAsVs,
    VSurfacePixelAsVsMut, VSurfacePixelMut, VSurfaceRail, VSurfaceRailAsVs, VSurfaceRailAsVsMut,
    VSurfaceRailMut,
};
use facto_loop_miner_fac_engine::common::varea::VArea;
use facto_loop_miner_fac_engine::common::vpoint::VPoint;
use facto_loop_miner_fac_engine::common::vpoint_direction::{VPointDirectionQ, VSegment};
use facto_loop_miner_fac_engine::game_blocks::rail_hope::RailHopeLink;
use facto_loop_miner_fac_engine::game_blocks::rail_hope_single::SECTION_POINTS_I32;
use facto_loop_miner_fac_engine::game_entities::direction::FacDirectionQuarter;
use itertools::Itertools;
use simd_json::prelude::ArrayTrait;
use std::collections::HashSet;
use std::ops::ControlFlow;
use std::rc::Rc;
use tracing::{error, info, trace, warn};

const BATCH_SIZE_MAX: usize = 3;

/// Planner v2 "Regis Altare 🎇"
///
/// Pathfinding with medium-difficulty backtracking.
/// because v0 Mori and v1 Ruze Planner can mask valid routes
pub fn start_altare_planner(
    tunables: &PathingTunables,
    surface_mut: VSurfaceNavMut,
    params: &StepParams,
) {
    /////
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
    all_mine_locations_main: Vec<MineLocation>,
    base_source_positive: Rc<RefCell<BaseSourceEighth>>,
    origin_base: VPointDirectionQ,
    origin_index: i32,
    origin_sign_pos: bool,
    is_prev_retry: bool,
    tunables: &'t PathingTunables,
}

impl<'t, 'sr, 's> Quester<'t, 'sr, 's> {
    fn init(tunables: &'t PathingTunables, surface: &'sr mut VSurfaceNavMut<'s>) -> Self {
        let base_source = BaseSource::from_central_base(tunables).into_refcells();
        let base_source_positive = base_source.positive_rc();

        let all_mine_locations_main = group_nearby_patches(surface.patches());
        draw_prep_mines(
            &mut surface.pixels_mut(),
            &all_mine_locations_main,
            &base_source_positive,
        );

        Quester {
            surface,
            all_mine_locations_main,
            base_source_positive,
            origin_base: VPointDirectionQ(VPoint::new(0, 0), FacDirectionQuarter::East),
            origin_index: 0,
            origin_sign_pos: true,
            is_prev_retry: false,
            tunables,
        }
    }
    //
    // fn dummy_start(&mut self) {
    //     let surface_rails = &mut surface.rails_mut();
    //     let QuesterScanResult::NewPatchesInScanArea(patches) = self.scan_patches() else {
    //         unimplemented!()
    //     };
    //
    //     if let ControlFlow::Break(()) = self.new_patches_in_scan_area(patches) {}
    // }

    fn start(&mut self, surface: &mut VSurfaceNavMut) {
        let mut limiter_counter = 0;
        loop {
            match self.scan_patches() {
                QuesterScanResult::YAxisEnding => {
                    info!("end of processing");
                    break;
                }
                QuesterScanResult::NoPatchesInScan => {
                    self.origin_index += 1;
                    continue;
                }
                QuesterScanResult::NewPatchesInScanArea(patches) => {
                    if limiter_counter >= 99999 {
                        self.debug_iteration(surface, limiter_counter);
                        break;
                    }
                    limiter_counter += 1;
                    self.new_patches_in_scan_area(patches)
                }
            }
        }

        info!("post save to gif buffering");
        for _ in 0..4 {
            surface
                .pixels()
                .paint_pixel_colored_zoomed()
                .save_to_oculante();
        }
    }

    fn scan_patches(&mut self) -> QuesterScanResult {
        let surface = self.surface.rails();
        let scan_sign = if self.origin_sign_pos { 1 } else { -1 };
        let scan_start = self.origin_base.point().move_direction_sideways_int(
            self.origin_base.direction(),
            PERPENDICULAR_SCAN_WIDTH * self.origin_index * scan_sign,
        );
        let scan_end = {
            let mut pos = scan_start.move_direction_sideways_int(
                self.origin_base.direction(),
                PERPENDICULAR_SCAN_WIDTH,
            );
            if !pos.is_within_center_radius(surface.pixels().get_radius()) {
                return QuesterScanResult::YAxisEnding;
            }
            loop {
                let next = pos.move_direction_int(self.origin_base.direction(), 1);
                if pos.is_within_center_radius(surface.pixels().get_radius()) {
                    pos = next;
                } else {
                    break;
                }
            }
            pos
        };
        let scan_area = VArea::from_arbitrary_points_pair(&scan_start, &scan_end);

        let already_pathed_mines: Vec<&MineLocation> = surface
            .get_mine_paths()
            .into_iter()
            .map(|v| &v.location)
            .collect();
        let mut new_patches_in_scan_area: Vec<&MineLocation> = self
            .all_mine_locations_main
            .iter()
            .filter(|v| {
                !already_pathed_mines.contains(&v)
                    && scan_area.contains_point(&v.area_min().point_center())
            })
            .collect();
        if new_patches_in_scan_area.is_empty() {
            warn!("no mines found in {}", scan_area);
            return QuesterScanResult::NoPatchesInScan;
        }
        new_patches_in_scan_area.sort_by(|a, b| {
            VPoint::sort_by_y_then_x_row(a.area_min().point_center(), b.area_min().point_center())
        });

        info!(
            "discovered {} patches in {scan_area}",
            new_patches_in_scan_area.len()
        );

        self.new_patches_in_scan_area(new_patches_in_scan_area)
            .is_break();
        // QuesterScanResult::NewPatchesInScanArea(new_patches_in_scan_area)
        todo!()
    }

    fn debug_iteration(&self, surface: &mut VSurfaceNavMut, limiter_counter: u32) {
        // best = 16
        // better = 28, 30, 32
        info!("limiter {limiter_counter}");
        // break;
        let base_source = self.base_source_positive.borrow_mut().next().unwrap();
        let start = base_source.origin;
        let end = VPointDirectionQ(
            VPoint::new(SECTION_POINTS_I32 * 100, SECTION_POINTS_I32 * 100),
            FacDirectionQuarter::East,
        );
        let surface = surface.pixels();

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
    // }
    //
    // struct QuesterProcessor<'t, 'sr, 's, 'q> {
    //     quester: &'q Quester<'t, 'sr, 's>,
    //     mines_all: &'q mut Vec<MineLocation>,
    //     mines_remain: Vec<&'q MineLocation>,
    //     base_source_positive: Rc<RefCell<BaseSourceEighth>>,
    // }
    //
    // impl<'t, 'sr, 's, 'q> QuesterProcessor<'t, 'sr, 's, 'q> {
    fn new_patches_in_scan_area(&mut self, new_patches: Vec<&MineLocation>) -> ControlFlow<()> {
        // let buffer_areas: Vec<RemovedEntity> = mines
        //     .iter()
        //     .map(|p| p.draw_area_buffered_to_no_touch(surface))
        //     .collect();
        let mines = self.fill_queue(new_patches);
        let possible_routes = get_possible_routes_for_batch(
            self.surface.pixels(),
            MineSelectBatch {
                base_sources: self.base_source_positive.clone(),
                mines,
            },
        );
        // info!("batch has {} sequences", possible_routes.sequences.len());

        self.execute_plan(possible_routes)
    }

    fn fill_queue(&mut self, mut new_patches: Vec<&MineLocation>) -> Vec<MineLocation> {
        let mut mines: Vec<MineLocation> = Vec::new();
        for _ in 0..BATCH_SIZE_MAX.saturating_sub(1) {
            if let Some((mine, removed_points)) = self.surface.rails_mut().remove_mine_path_pop() {
                trace!("batch pop from mine {BATCH_SIZE_MAX}");
                MineLocation::restore_area_buffered(
                    &[&mine.location],
                    &mut self.surface.pixels_mut(),
                    removed_points,
                );
                mines.push(mine.location);
                self.base_source_positive.borrow_mut().undo_one();
            }
        }
        while mines.len() != BATCH_SIZE_MAX {
            trace!("batch pop from patches");
            mines.push(new_patches.pop().unwrap().clone());
        }
        assert_eq!(mines.len(), BATCH_SIZE_MAX);
        mines
    }

    fn execute_plan(&mut self, possible_routes: CompletePlan) -> ControlFlow<()> {
        let surface = self.surface;
        match execute_route_batch_clone_prep(
            self.tunables.mori(),
            &mut surface.pixels_mut(),
            possible_routes.sequences,
            &[ExecuteFlags::ShrinkBases],
        ) {
            ExecutorResult::Success { paths, routes } => {
                self.is_prev_retry = false;
                self.base_source_positive
                    .borrow_mut()
                    .advance_by(paths.len())
                    .unwrap();
                // routes.last().unwrap().location.draw_area_buffered(surface);
                for path in paths {
                    surface.rails_mut().add_mine_path(path);
                }
                surface
                    .pixels()
                    .paint_pixel_colored_zoomed()
                    .save_to_oculante();
                ControlFlow::Continue(())
            }
            ExecutorResult::Failure { meta, seen_mines } => {
                // || is_prev_retry todo
                if surface.rails().get_mine_paths().is_empty() {
                    error!("failed to pathfind! but no rollback after another rollback");
                    debug_failing(&mut surface.rails_mut(), meta);
                    ControlFlow::Break(())
                } else {
                    self.is_prev_retry = true;
                    info!("attempting retry");
                    assert!(
                        !surface.rails().get_mine_paths().is_empty(),
                        "too early to retry"
                    );

                    if meta.all_routes.len() == seen_mines.len() {
                        debug_draw_failing_mines(&mut surface.pixels_mut(), &seen_mines);

                        surface
                            .pixels()
                            .paint_pixel_colored_zoomed()
                            .save_to_oculante();
                        error!("combination of {} mines cannot be found", seen_mines.len());
                        return ControlFlow::Break(());
                    }
                    assert_ne!(meta.all_routes.len(), seen_mines.len());

                    let all_mines = meta
                        .all_routes
                        .into_iter()
                        .map(|v| v.location)
                        .collect_vec();

                    let mut never_mined = all_mines
                        .into_iter()
                        .filter(|all_mine| !seen_mines.contains(all_mine))
                        .collect_vec();
                    if never_mined.len() != 1 {
                        error!("never_mined actual {} expected {}", never_mined.len(), 1);
                        return ControlFlow::Break(());
                    }
                    assert_eq!(never_mined.len(), 1);
                    let never_mined = never_mined.remove(0);

                    // where tf are we
                    // surface.draw_square_area_forced(
                    //     &VArea::from_radius(never_mined.area_min().point_center(), 20)
                    //         .normalize_within_radius(surface.get_radius_i32() - 1),
                    //     Pixel::Highlighter,
                    // );

                    let nearest_rail = detect_nearby_rails_as_index(surface.rails(), &never_mined);
                    rollback_and_reapply(
                        &mut surface.rails_mut(),
                        self.tunables,
                        nearest_rail,
                        never_mined,
                        &mut self.base_source_positive.borrow_mut(),
                        &self.all_mine_locations_main,
                    );

                    surface
                        .pixels()
                        .paint_pixel_colored_zoomed()
                        .save_to_oculante();
                    // we may took another attempt

                    let scan_sign = if self.origin_sign_pos { 1 } else { -1 };
                    self.origin_index -= match self.origin_index.unsigned_abs() {
                        3.. => scan_sign * 3,
                        2 => scan_sign * 2,
                        1 => scan_sign,
                        0 => 0,
                    };

                    if self.origin_index > 1 {
                        self.origin_index -= 1;
                    } else if self.origin_index < 1 {
                        self.origin_index += 1;
                    }

                    ControlFlow::Continue(())
                }
            }
        }
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
        .into_iter()
        .position(|p| {
            p.links
                .iter()
                .any(|link| link.area_vec().contains(&closest_rail))
        })
        .unwrap_or_else(|| panic!("No rail found at {closest_rail}"))
}

fn rollback_and_reapply(
    surface: &mut VSurfaceRailMut,
    tunables: &PathingTunables,
    old_rail_index: usize,
    new_mine: MineLocation,
    base_source: &mut BaseSourceEighth,
    all_mines: &[MineLocation],
) {
    // remove old rail
    let (old_path, _) = surface.remove_mine_path_at(old_rail_index).unwrap();

    // re-pathfind with restricted barriers. this SHOULD succeed
    let mut base_source_dummy = base_source.regenerate();
    while base_source_dummy.peek_single().origin != old_path.segment.start {
        base_source_dummy.next();
    }
    let plan = get_possible_routes_for_batch(
        surface.pixels(),
        MineSelectBatch {
            base_sources: base_source_dummy.into_rc_refcell(),
            mines: vec![new_mine],
        },
    );
    assert!(!plan.sequences.is_empty());
    for sequence in &plan.sequences {
        assert_eq!(sequence.routes.len(), 1);
        trace!(
            "plan sequence location {:?} segment {}",
            sequence.routes[0].location, sequence.routes[0].segment
        )
    }
    // assert_eq!(plan.sequences.len(), 1);
    // assert_eq!(plan.sequences[0].routes.len(), 1);
    let new_path = match execute_route_batch_clone_prep(
        tunables.mori(),
        &mut surface.pixels_mut(),
        plan.sequences,
        &[ExecuteFlags::ShrinkBases],
    ) {
        ExecutorResult::Failure { meta, .. } => {
            debug_failing(surface, meta);
            surface.add_mine_path_with_pixel(old_path, Pixel::Highlighter);
            surface
                .pixels()
                .paint_pixel_colored_zoomed()
                .save_to_oculante();
            panic!("uhh")
        }
        ExecutorResult::Success { mut paths, routes } => {
            assert_eq!(paths.len(), 1);
            paths.remove(0)
        }
    };

    // re-pathfind everything after
    while surface.rails().get_mine_paths().len() != old_rail_index {
        info!(
            "mines len {} or {old_rail_index}",
            surface.rails().get_mine_paths().len()
        );
        let (_, removed_points) = surface.remove_mine_path_pop().unwrap();
        MineLocation::restore_area_buffered(all_mines, &mut surface.pixels_mut(), removed_points);
        base_source.undo_one();
    }
    surface
        .pixels()
        .paint_pixel_colored_zoomed()
        .save_to_oculante();
    surface.add_mine_path(new_path);
}

enum QuesterScanResult<'s> {
    YAxisEnding,
    NoPatchesInScan,
    NewPatchesInScanArea(Vec<&'s MineLocation>),
}
