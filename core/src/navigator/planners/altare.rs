use crate::navigator::base_source::BaseSource;
use crate::navigator::mine_executor::{execute_route_batch, MineRouteCombinationPathResult};
use crate::navigator::mine_permutate::get_possible_routes_for_batch;
use crate::navigator::mine_selector::{
    group_nearby_patches, MineSelectBatch, PERPENDICULAR_SCAN_WIDTH,
};
use crate::navigator::planners::common::{draw_prep, draw_prep_mines};
use crate::state::machine::StepParams;
use crate::surfacev::mine::MineLocation;
use crate::surfacev::vpatch::VPatch;
use crate::surfacev::vsurface::{RemovedEntity, VSurface};
use facto_loop_miner_common::err_bt::pretty_print_error;
use facto_loop_miner_fac_engine::admiral::lua_command::fac_surface_create_tile::FacSurfaceCreateLua;
use facto_loop_miner_fac_engine::common::varea::VArea;
use facto_loop_miner_fac_engine::common::vpoint::VPoint;
use facto_loop_miner_fac_engine::common::vpoint_direction::VPointDirectionQ;
use facto_loop_miner_fac_engine::game_entities::direction::FacDirectionQuarter;
use simd_json::prelude::ArrayTrait;
use std::sync::Mutex;
use tracing::{debug, error, info, warn};

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

    loop {
        match quester.scan_patches(&surface) {
            QuesterScanResult::YAxisEnding => {
                info!("end of processing");

                break;
            }
            QuesterScanResult::NoPatchesInScan => {
                quester.origin_index += 1;
            }
            QuesterScanResult::NewPatchesInScanArea(patches) => {
                let buffer_areas: Vec<RemovedEntity> = patches
                    .iter()
                    .map(|p| p.draw_area_buffered_to_no_touch(surface))
                    .collect();
                let possible_routes = get_possible_routes_for_batch(
                    surface,
                    MineSelectBatch {
                        base_sources: base_source_positive.clone(),
                        mines: patches.into_iter().take(BATCH_SIZE_MAX).cloned().collect(),
                    },
                );
                info!("found {} sequences", possible_routes.sequences.len());

                if let Err(e) = surface.load_clone_prep(&params.step_out_dir) {
                    pretty_print_error(e);
                    panic!("uhh");
                }
                let route_result = execute_route_batch(surface, possible_routes.sequences);
                match route_result {
                    MineRouteCombinationPathResult::Success { paths, routes } => {
                        base_source_positive
                            .borrow_mut()
                            .advance_by(paths.len())
                            .unwrap();
                        for buffer_area in buffer_areas {
                            MineLocation::draw_area_no_touch_to_buffered(surface, buffer_area);
                        }
                        for path in paths {
                            surface.add_mine_path(path).unwrap();
                        }
                        surface.save_pixel_to_oculante();
                    }
                    MineRouteCombinationPathResult::Failure { meta } => {
                        error!("failed to pathfind!");
                        break;
                    }
                }
            }
        }
    }
}

const BATCH_SIZE_MAX: usize = 4;

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
            VPoint::sort_by_direction(
                *self.origin_base.direction(),
                a.area_min().point_center(),
                b.area_min().point_center(),
            )
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
