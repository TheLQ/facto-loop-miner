use crate::navigator::base_source::{BaseSource, BaseSourceEighth};
use crate::navigator::mine_executor::{execute_route_batch, ExecuteFlags, ExecutorResult};
use crate::navigator::mine_permutate::get_possible_routes_for_batch;
use crate::navigator::mine_selector::{
    group_nearby_patches, MineSelectBatch, PERPENDICULAR_SCAN_WIDTH,
};
use crate::navigator::planners::common::{
    debug_draw_failing_mines, debug_failing, draw_prep, draw_prep_mines,
};
use crate::state::machine::StepParams;
use crate::surface::pixel::Pixel;
use crate::surfacev::mine::MineLocation;
use crate::surfacev::vpatch::VPatch;
use crate::surfacev::vsurface::{RemovedEntity, VSurface};
use facto_loop_miner_common::err_bt::pretty_print_error;
use facto_loop_miner_fac_engine::admiral::lua_command::fac_surface_create_tile::FacSurfaceCreateLua;
use facto_loop_miner_fac_engine::common::varea::VArea;
use facto_loop_miner_fac_engine::common::vpoint::{VPoint, VPOINT_ZERO};
use facto_loop_miner_fac_engine::common::vpoint_direction::VPointDirectionQ;
use facto_loop_miner_fac_engine::game_blocks::rail_hope::RailHopeLink;
use facto_loop_miner_fac_engine::game_entities::direction::FacDirectionQuarter;
use itertools::Itertools;
use simd_json::prelude::ArrayTrait;
use std::sync::Mutex;
use strum::VariantArray;
use tracing::{debug, error, info, trace, warn};

const BATCH_SIZE_MAX: usize = 1;

pub fn start_altare_planner(surface: &mut VSurface, params: &StepParams) {
    let base_source = BaseSource::from_central_base(&surface).into_refcells();
    let base_source_positive = base_source.positive_rc();

    let all_mine_locations = group_nearby_patches(surface);
    draw_prep_mines(surface, &all_mine_locations, &base_source_positive);
    let mut quester = Quester {
        all_mine_locations,
        origin_base: VPointDirectionQ(VPoint::new(0, 0), FacDirectionQuarter::East),
        origin_index: 0,
        origin_sign_pos: true,
    };

    // let mut limiter_counter = 0;
    let mut is_prev_retry = false;
    loop {
        match quester.scan_patches(&surface) {
            QuesterScanResult::YAxisEnding => {
                info!("end of processing");

                break;
            }
            QuesterScanResult::NoPatchesInScan => {
                quester.origin_index += 1;
            }
            QuesterScanResult::NewPatchesInScanArea(mut patches) => {
                // if limiter_counter >= 2 {
                //     info!("limiter {limiter_counter}");
                //     break;
                // }
                // limiter_counter += 1;
                let mut mines: Vec<MineLocation> = Vec::new();

                for _ in 0..BATCH_SIZE_MAX.saturating_sub(1) {
                    if let Some(mine) = surface.remove_mine_path_pop() {
                        trace!("batch pop from mine {BATCH_SIZE_MAX}");
                        mines.push(mine.mine_base);
                        base_source_positive.borrow_mut().undo_one();
                    }
                }
                while mines.len() != BATCH_SIZE_MAX {
                    trace!("batch pop from patches");
                    mines.push(patches.pop().unwrap().clone());
                }
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
                info!("found {} sequences", possible_routes.sequences.len());

                // if let Err(e) = surface.load_clone_prep(&params.step_out_dir) {
                //     pretty_print_error(e);
                //     panic!("uhh");
                // }
                let route_result = execute_route_batch(
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
                        routes.last().unwrap().location.draw_area_buffered(surface);
                        for path in paths {
                            surface.add_mine_path(path).unwrap();
                        }
                        surface.save_pixel_to_oculante_zoomed();
                    }
                    ExecutorResult::Failure { meta, seen_mines } => {
                        if false && is_prev_retry {
                            error!("failed to pathfind! but no rollback after another rollback");
                            debug_failing(surface, meta);
                            break;
                        } else {
                            is_prev_retry = true;
                            info!("attempting retry");

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
                            assert_eq!(never_mined.len(), 1);
                            let never_mined = never_mined.remove(0);

                            // where tf are we
                            surface.draw_square_area_forced(
                                &VArea::from_radius(never_mined.area_min().point_center(), 20)
                                    .normalize_within_radius(surface.get_radius_i32() - 1),
                                Pixel::Highlighter,
                            );

                            let nearest_rail = detect_nearby_rails_as_index(surface, &never_mined);
                            rollback_and_reapply(
                                surface,
                                nearest_rail,
                                never_mined,
                                &base_source_positive.borrow(),
                            );

                            surface.save_pixel_to_oculante_zoomed();
                        }
                    }
                }
            }
        }
    }
}

fn detect_nearby_rails_as_index(surface: &VSurface, mine_location: &MineLocation) -> usize {
    let origin = mine_location
        .area_min()
        .point_center()
        .move_round_even_down();
    origin.assert_even_position();

    let mut closest_rail = None;
    for direction in FacDirectionQuarter::VARIANTS {
        for depth in 0.. {
            let cursor = origin.move_direction_int(direction, depth * 2);
            if surface.is_point_out_of_bounds(&cursor) {
                trace!("no rail at {depth} direction {direction}");
                break;
            }
            match surface.get_pixel(cursor) {
                Pixel::Empty | Pixel::MineNoTouch | Pixel::Highlighter => {
                    // the vast expanse...
                }
                Pixel::Rail => {
                    closest_rail = match closest_rail {
                        None => Some((cursor, depth)),
                        Some((prev_cursor, prev_depth)) if depth < prev_depth => {
                            Some((cursor, depth))
                        }
                        Some(good) => Some(good),
                    };
                    trace!("found rail at {depth} direction {direction}");
                    break;
                }
                pixel if Pixel::is_resource(&pixel) => {
                    // ignore resources
                }
                pixel => {
                    // resource buffer area probably
                    trace!("hit limit at depth {depth} at {pixel:?} direction {direction}");
                    break;
                }
            }
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
    base_source: &BaseSourceEighth,
) {
    // remove old rail
    let old_path = surface.remove_mine_path_at(old_rail_index).unwrap();

    // re-pathfind with restricted barriers. this SHOULD succeed
    let mut base_source_dummy = base_source.regenerate();
    while base_source_dummy.peek_single().origin != old_path.segment.start {
        base_source_dummy.next();
    }
    let mut plan = get_possible_routes_for_batch(
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
            sequence.routes[0].location,
            sequence.routes[0].segment
        )
    }
    // assert_eq!(plan.sequences.len(), 1);
    // assert_eq!(plan.sequences[0].routes.len(), 1);
    let new_path = match execute_route_batch(surface, plan.sequences, &[ExecuteFlags::ShrinkBases])
    {
        ExecutorResult::Failure { meta, .. } => {
            debug_failing(surface, meta);
            // surface
            //     .add_mine_path_with_pixel(old_path, Pixel::Highlighter)
            //     .unwrap();
            surface.save_pixel_to_oculante_zoomed();
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
        surface.remove_mine_path_pop().unwrap();
    }
    surface.add_mine_path(new_path).unwrap();
}

struct Quester {
    all_mine_locations: Vec<MineLocation>,
    origin_base: VPointDirectionQ,
    origin_index: i32,
    origin_sign_pos: bool,
}

impl Quester {
    fn scan_patches(&self, surface: &VSurface) -> QuesterScanResult {
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
