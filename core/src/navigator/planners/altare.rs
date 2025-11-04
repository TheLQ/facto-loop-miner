use crate::navigator::base_source::{BaseSource, BaseSourceEighth};
use crate::navigator::circleify::draw_circle_around;
use crate::navigator::mine_executor::{
    ExecuteFlags, ExecutorResult, execute_route_batch, execute_route_batch_clone_prep,
};
use crate::navigator::mine_permutate::get_possible_routes_for_batch;
use crate::navigator::mine_selector::{
    MineSelectBatch, PERPENDICULAR_SCAN_WIDTH, group_nearby_patches,
};
use crate::navigator::mori::{MoriResult, count_link_origins, mori2_start};
use crate::navigator::planners::common::{
    debug_draw_failing_mines, debug_draw_mine_index_labels, debug_draw_mine_links, debug_failing,
    draw_prep_mines,
};
use crate::state::machine::StepParams;
use crate::state::tuneables::Tunables;
use crate::surface::pixel::Pixel;
use crate::surfacev::mine::MineLocation;
use crate::surfacev::vsurface::{VSurface, VSurfacePixelPatches};
use facto_loop_miner_fac_engine::common::varea::VArea;
use facto_loop_miner_fac_engine::common::vpoint::VPoint;
use facto_loop_miner_fac_engine::common::vpoint_direction::{VPointDirectionQ, VSegment};
use facto_loop_miner_fac_engine::game_blocks::rail_hope::RailHopeLink;
use facto_loop_miner_fac_engine::game_blocks::rail_hope_single::SECTION_POINTS_I32;
use facto_loop_miner_fac_engine::game_entities::direction::FacDirectionQuarter;
use itertools::Itertools;
use simd_json::prelude::ArrayTrait;
use std::collections::HashSet;
use tracing::{error, info, trace, warn};

const BATCH_SIZE_MAX: usize = 3;

/// Planner v2 "Regis Altare 🎇"
///
/// Pathfinding with medium-difficulty backtracking.
/// because v0 Mori and v1 Ruze Planner can mask valid routes
pub fn start_altare_planner(
    tunables: &Tunables,
    surface: VSurfacePixelPatches,
    params: &StepParams,
) {
    let base_source = BaseSource::from_central_base(tunables).into_refcells();
    let base_source_positive = base_source.positive_rc();

    let mut all_mine_locations = group_nearby_patches(surface);
    draw_prep_mines(surface, &all_mine_locations, &base_source_positive);

    /////

    // for mine in &mut all_mine_locations {
    //     mine.revalidate_endpoints_after_no_touch(surface);
    // }

    // debug_draw_mine_index_labels(surface, &all_mine_locations);
    // debug_draw_mine_links(surface, &all_mine_locations);
    // surface.paint_pixel_colored_entire().save_to_oculante();

    // let needle_mine = &all_mine_locations[62];
    // let debug_patches = needle_mine
    //     .surface_patches(surface)
    //     .map(|patch| DebugMinePatch {
    //         pixel: patch.resource,
    //         points: patch.pixel_indexes.clone(),
    //     })
    //     .collect_vec();
    // let converted = simd_json::to_string(&debug_patches).unwrap();
    // let path = Path::new("example_mine.json");
    // write_entire_file(path, converted.as_bytes())
    //     .convert(path)
    //     .unwrap();
    //
    // if crate::always_true_test() {
    //     return;
    // }

    /////

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

    ////

    // let radius = surface.get_radius_i32();
    // let mut grid = Vec::new();
    // for x in -radius..radius {
    //     for y in -radius..radius {
    //         if x % SECTION_POINTS_I32 == 0 || y % SECTION_POINTS_I32 == 0 {
    //             grid.push(VPoint::new(x, y));
    //         }
    //     }
    // }
    // surface.change_pixels(grid).stomp(Pixel::Highlighter);

    ////

    // if crate::always_true_test() {
    //     return;
    // }

    let mut quester = Quester {
        all_mine_locations,
        origin_base: VPointDirectionQ(VPoint::new(0, 0), FacDirectionQuarter::East),
        origin_index: 0,
        origin_sign_pos: true,
    };

    let mut limiter_counter = 0;
    let mut is_prev_retry = false;
    loop {
        match quester.scan_patches(surface) {
            QuesterScanResult::YAxisEnding => {
                info!("end of processing");

                break;
            }
            QuesterScanResult::NoPatchesInScan => {
                quester.origin_index += 1;
            }
            QuesterScanResult::NewPatchesInScanArea(mut patches) => {
                // DEBUG - Generate partial graduated images
                if limiter_counter >= 99999 {
                    // best = 16
                    // better = 28, 30, 32
                    info!("limiter {limiter_counter}");
                    break;
                    let base_source = base_source_positive.borrow_mut().next().unwrap();
                    let start = base_source.origin;
                    let end = VPointDirectionQ(
                        VPoint::new(SECTION_POINTS_I32 * 100, SECTION_POINTS_I32 * 100),
                        FacDirectionQuarter::East,
                    );

                    let fixed_radius = surface.get_radius_i32();
                    let fixed_finding_limiter = VArea::from_arbitrary_points_pair(
                        VPoint::new(0, -fixed_radius),
                        // Must give spacing from Edge, because hope_link.area() can extend past it.
                        // range checks are disabled for theoretical performance
                        VPoint::new(fixed_radius, fixed_radius),
                    );

                    let result =
                        mori2_start(surface, VSegment { start, end }, &fixed_finding_limiter);
                    let MoriResult::FailingDebug { err } = result else {
                        panic!("it worked? {end}")
                    };
                    surface
                        .paint_pixel_graduated(count_link_origins(&err.seen))
                        .save_to_oculante();

                    break;
                }
                limiter_counter += 1;
                let mut mines: Vec<MineLocation> = Vec::new();

                for _ in 0..BATCH_SIZE_MAX.saturating_sub(1) {
                    if let Some((mine, removed_points)) = surface.remove_mine_path_pop() {
                        trace!("batch pop from mine {BATCH_SIZE_MAX}");
                        MineLocation::restore_area_buffered(
                            &quester.all_mine_locations,
                            surface,
                            removed_points,
                        );
                        mines.push(mine.mine_base);
                        base_source_positive.borrow_mut().undo_one();
                    }
                }
                while mines.len() != BATCH_SIZE_MAX {
                    trace!("batch pop from patches");
                    mines.push(patches.pop().unwrap().clone());
                }
                assert_eq!(mines.len(), BATCH_SIZE_MAX);
                drop(patches);

                // let buffer_areas: Vec<RemovedEntity> = mines
                //     .iter()
                //     .map(|p| p.draw_area_buffered_to_no_touch(surface))
                //     .collect();
                let possible_routes = get_possible_routes_for_batch(
                    surface,
                    MineSelectBatch {
                        base_sources: base_source_positive.clone(),
                        mines,
                    },
                );
                // info!("batch has {} sequences", possible_routes.sequences.len());

                let route_result = execute_route_batch_clone_prep(
                    surface,
                    possible_routes.sequences,
                    &[ExecuteFlags::ShrinkBases],
                );
                match route_result {
                    ExecutorResult::Success { paths, routes } => {
                        is_prev_retry = false;
                        base_source_positive
                            .borrow_mut()
                            .advance_by(paths.len())
                            .unwrap();
                        // routes.last().unwrap().location.draw_area_buffered(surface);
                        for path in paths {
                            surface.add_mine_path(path);
                        }
                        surface.paint_pixel_colored_zoomed().save_to_oculante();
                    }
                    ExecutorResult::Failure { meta, seen_mines } => {
                        // || is_prev_retry todo
                        if surface.get_mine_paths().is_empty() {
                            error!("failed to pathfind! but no rollback after another rollback");
                            debug_failing(surface, meta);
                            break;
                        } else {
                            is_prev_retry = true;
                            info!("attempting retry");
                            assert!(!surface.get_mine_paths().is_empty(), "too early to retry");

                            if meta.all_routes.len() == seen_mines.len() {
                                debug_draw_failing_mines(surface, &seen_mines);

                                surface.paint_pixel_colored_zoomed().save_to_oculante();
                                error!("combination of {} mines cannot be found", seen_mines.len());
                                break;
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
                                break;
                            }
                            assert_eq!(never_mined.len(), 1);
                            let never_mined = never_mined.remove(0);

                            // where tf are we
                            // surface.draw_square_area_forced(
                            //     &VArea::from_radius(never_mined.area_min().point_center(), 20)
                            //         .normalize_within_radius(surface.get_radius_i32() - 1),
                            //     Pixel::Highlighter,
                            // );

                            let nearest_rail = detect_nearby_rails_as_index(surface, &never_mined);
                            rollback_and_reapply(
                                surface,
                                nearest_rail,
                                never_mined,
                                &mut base_source_positive.borrow_mut(),
                                &quester.all_mine_locations,
                            );

                            surface.paint_pixel_colored_zoomed().save_to_oculante();
                            // we may took another attempt

                            let scan_sign = if quester.origin_sign_pos { 1 } else { -1 };
                            quester.origin_index -= match quester.origin_index.unsigned_abs() {
                                3.. => scan_sign * 3,
                                2 => scan_sign * 2,
                                1 => scan_sign,
                                0 => 0,
                            };

                            if quester.origin_index > 1 {
                                quester.origin_index -= 1;
                            } else if quester.origin_index < 1 {
                                quester.origin_index += 1;
                            }
                        }
                    }
                }
            }
        }
    }

    info!("post save to gif buffering");
    for _ in 0..4 {
        surface.paint_pixel_colored_zoomed().save_to_oculante();
    }
}

fn detect_nearby_rails_as_index(surface: &VSurface, mine_location: &MineLocation) -> usize {
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
            if !cursor.is_even() || surface.is_point_out_of_bounds(&cursor) {
                continue;
            }

            if seen_points.contains(&cursor) {
                continue;
            }
            seen_points.insert(cursor);

            let distance = origin.distance_bird(&cursor).abs();
            match surface.get_pixel(cursor) {
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
    surface: &mut VSurface,
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
        surface,
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
    let new_path =
        match execute_route_batch_clone_prep(surface, plan.sequences, &[ExecuteFlags::ShrinkBases])
        {
            ExecutorResult::Failure { meta, .. } => {
                debug_failing(surface, meta);
                surface.add_mine_path_with_pixel(old_path, Pixel::Highlighter);
                surface.paint_pixel_colored_zoomed().save_to_oculante();
                panic!("uhh")
            }
            ExecutorResult::Success { mut paths, routes } => {
                assert_eq!(paths.len(), 1);
                paths.remove(0)
            }
        };

    // re-pathfind everything after
    while surface.get_mine_paths().len() != old_rail_index {
        info!(
            "mines len {} or {old_rail_index}",
            surface.get_mine_paths().len()
        );
        let (_, removed_points) = surface.remove_mine_path_pop().unwrap();
        MineLocation::restore_area_buffered(all_mines, surface, removed_points);
        base_source.undo_one();
    }
    surface.paint_pixel_colored_zoomed().save_to_oculante();
    surface.add_mine_path(new_path);
}

struct Quester {
    all_mine_locations: Vec<MineLocation>,
    origin_base: VPointDirectionQ,
    origin_index: i32,
    origin_sign_pos: bool,
}

impl Quester {
    fn scan_patches(&self, surface: &VSurface) -> QuesterScanResult<'_> {
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
            if !pos.is_within_center_radius(surface.get_radius()) {
                return QuesterScanResult::YAxisEnding;
            }
            loop {
                let next = pos.move_direction_int(self.origin_base.direction(), 1);
                if pos.is_within_center_radius(surface.get_radius()) {
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
            .map(|v| &v.mine_base)
            .collect();
        let mut new_patches_in_scan_area: Vec<&MineLocation> = self
            .all_mine_locations
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
        QuesterScanResult::NewPatchesInScanArea(new_patches_in_scan_area)
    }
}

enum QuesterScanResult<'q> {
    YAxisEnding,
    NoPatchesInScan,
    NewPatchesInScanArea(Vec<&'q MineLocation>),
}
