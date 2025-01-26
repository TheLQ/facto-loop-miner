use crate::navigator::mori::{Rail, RailDirection};
use crate::navigator::path_side::{BaseSource, BaseSourceEighth};
use crate::state::machine_v1::{CENTRAL_BASE_TILES, REMOVE_RESOURCE_BASE_TILES};
use crate::surface::patch::map_vpatch_to_kdtree;
use crate::surface::pixel::Pixel;
use crate::surfacev::vpatch::VPatch;
use crate::surfacev::vsurface::VSurface;
use crate::TILES_PER_CHUNK;
use facto_loop_miner_fac_engine::common::varea::VArea;
use facto_loop_miner_fac_engine::common::vpoint::VPoint;
use itertools::Itertools;
use kiddo::{Manhattan, NearestNeighbour};
use serde::{Deserialize, Serialize};
use simd_json::prelude::ArrayTrait;
use std::rc::Rc;
use std::sync::Mutex;
use tracing::{debug, error, info, trace};

const MAX_PATCHES: usize = 200;

pub struct MineBaseBatch {
    pub mines: Vec<MineBase>,
    pub base_source_eighth: Rc<Mutex<BaseSourceEighth>>,
    pub base_direction: RailDirection,
    pub batch_search_area: VArea,
}

pub enum MineBaseBatchResult {
    Success { batches: Vec<MineBaseBatch> },
    EmptyBatch { batch: MineBaseBatch },
}

impl MineBaseBatchResult {
    pub fn into_success(self) -> Option<Vec<MineBaseBatch>> {
        match self {
            MineBaseBatchResult::Success { batches } => Some(batches),
            MineBaseBatchResult::EmptyBatch { .. } => None,
        }
    }
}

pub const MAXIMUM_MINE_COUNT_PER_BATCH: usize = 5;
pub const RESPLIT_LAST_COUNT_LESS_THAN_THRESHOLD: usize = 3;
pub const PERPENDICULAR_SCAN_WIDTH: u32 = 20;

/// Solve these core problems
/// - Find the patches we care about
/// - Create batches that can be optimized as a group, because larger groups are exponentially more difficult to optimize
pub fn get_mine_bases_by_batch(
    surface: &VSurface,
    base_source: &BaseSource,
) -> MineBaseBatchResult {
    let patch_groups = group_nearby_patches(
        surface,
        &[Pixel::IronOre, Pixel::CopperOre, Pixel::Stone, Pixel::Coal],
    );

    // let ordered_patches = match 2 {
    //     1 => patches_by_radial_base_corner(surface, Pixel::IronOre),
    //     // 2 => patches_by_cross_sign_expanding(
    //     //     surface,
    //     //     &[Pixel::IronOre, Pixel::CopperOre, Pixel::Stone, Pixel::Coal],
    //     // ),
    //     _ => panic!("asd"),
    // };
    // ordered_patches

    let mine_batches = patches_by_cross_sign_expanding(patch_groups, base_source);
    // let mut result = Vec::new();
    // {
    //     let mut iter = mine_batches.into_iter();
    //     while let Some(v) = iter.next() {
    //
    //     }
    // }
    // let mine_batches = result;

    let mut result = Vec::new();
    for (index, mine_batch) in mine_batches.into_iter().enumerate() {
        // When expanded, 6! = 720. 9! = 362,880 which is too gigantic

        let mine_batch_len = mine_batch.mines.len();
        if mine_batch_len > MAXIMUM_MINE_COUNT_PER_BATCH {
            for chunk in mine_batch
                .mines
                .into_iter()
                .chunks(MAXIMUM_MINE_COUNT_PER_BATCH)
                .into_iter()
            {
                result.push(MineBaseBatch {
                    mines: chunk.into_iter().collect(),
                    base_source_eighth: mine_batch.base_source_eighth.clone(),
                    batch_search_area: mine_batch.batch_search_area.clone(),
                    base_direction: mine_batch.base_direction.clone(),
                })
            }
            // // avoid last chunk with eg 1 mine that
            // let (_, last_chunk) = result.split_last_chunk_mut::<2>().unwrap();
            // if last_chunk[1].mines.len() < RESPLIT_LAST_COUNT_LESS_THAN_THRESHOLD {
            //
            // }
        } else if mine_batch_len == 0 {
            error!("bad batch at {}", index);
            return MineBaseBatchResult::EmptyBatch { batch: mine_batch };
        } else {
            result.push(mine_batch);
        }
    }
    MineBaseBatchResult::Success { batches: result }
}

/// Second grouping pass (after opencv), now by grouping different resource patches
pub fn group_nearby_patches(surface: &VSurface, resources: &[Pixel]) -> Vec<MineBase> {
    let patches: Vec<&VPatch> = surface
        .get_patches_slice()
        .iter()
        .filter(|patch| resources.contains(&patch.resource))
        .filter(|patch| {
            !patch
                .area
                .start
                .is_within_center_radius(REMOVE_RESOURCE_BASE_TILES as u32)
        })
        .collect();
    let patch_range = 0..patches.len();

    // group patches by neary
    let mut groups: Vec<Vec<&VPatch>> = Vec::new();
    for patch in &patches {
        let processed_patches = groups.iter().flatten().cloned().collect_vec();
        if processed_patches.contains(patch) {
            // already in a group
            continue;
        }
        let remaining_patches = patches
            .iter()
            .filter(|p| !processed_patches.contains(p))
            .cloned()
            .collect_vec();

        let mut new_group = Vec::new();
        new_group.push(*patch);
        recursive_near_patches(patch, &remaining_patches, &mut new_group);
        if new_group.len() != 1 {
            info!("group of {}", new_group.len());
        }
        groups.push(new_group);
    }

    // {
    //     let mut dedupe_check = groups.iter().flatten().cloned().collect_vec();
    //     let old = dedupe_check.len();
    //     dedupe_check.sort();
    //     dedupe_check.dedup();
    //     assert_eq!(old, dedupe_check.len(), "dedupe found stuff!");
    // }

    // Merge groups
    let mut result = Vec::new();
    for patch_group in groups {
        if patch_group.len() != 1 {
            trace!("Merging patch group of {:?}", patch_group);

            // Externally we use the index in the VSurface Patches slice
            let patch_group_indexes = patch_group
                .iter()
                .map(|patch| patch.get_surface_patch_index(surface))
                .collect();

            let area = VArea::from_arbitrary_points(
                patch_group.iter().flat_map(|patch| &patch.pixel_indexes),
            );

            result.push(MineBase {
                patch_indexes: patch_group_indexes,
                area,
            });
            // panic!("TODO: Broken area");
        } else {
            let patch = patch_group[0];
            trace!("Single patch group {:?}", patch);
            result.push(MineBase {
                patch_indexes: vec![patch.get_surface_patch_index(surface)],
                area: patch.area.clone(),
            })
        }
    }
    result
}

fn recursive_near_patches<'a>(
    needle: &VPatch,
    patches: &[&'a VPatch],
    total: &mut Vec<&'a VPatch>,
) {
    for other in patches {
        if *other == needle || total.contains(other) {
            continue;
        }
        if needle
            .area
            .point_center()
            .distance_bird(&other.area.point_center())
            < TILES_PER_CHUNK as f32 * 4.0
        {
            total.push(*other);
            recursive_near_patches(other, patches, total);
        }
    }
}

// #[allow(clippy::never_loop)]
fn patches_by_cross_sign_expanding(
    mut mines: Vec<MineBase>,
    base_source: &BaseSource,
) -> Vec<MineBaseBatch> {
    let bounding_area =
        VArea::from_arbitrary_points(mines.iter().flat_map(|v| v.area.get_corner_points()));
    let cross_sides: [Rail; 1] = [
        // Rail::new_straight(
        //     VPoint::new(REMOVE_RESOURCE_BASE_TILES, 0),
        //     RailDirection::Right,
        // )
        Rail::new_straight(
            VPoint::new(REMOVE_RESOURCE_BASE_TILES, 0),
            RailDirection::Right,
        ),
    ];
    let mut batches = Vec::new();
    for cross_side in cross_sides {
        for perpendicular_scan_size_base in (1i32..).flat_map(|i| [i, -i]) {
            // if perpendicular_scan_area.unsigned_abs() * RAIL_STEP_SIZE > surface.get_radius() {
            //     break;
            // }

            // TODO: Support multiple sides
            let base_source_eighth = if perpendicular_scan_size_base > 0 {
                // TODO: While going positive we get a... negative position?
                base_source.get_negative()
            } else {
                base_source.get_positive()
            };

            let scan_start = {
                let mut rail = cross_side.clone();
                // trace!("start moving forward {}", parallel_scan_area - 1);
                rail = if perpendicular_scan_size_base > 0 {
                    rail.move_force_rotate_clockwise(1)
                } else {
                    rail.move_force_rotate_clockwise(3)
                };
                rail = rail.move_forward_step_num(
                    (perpendicular_scan_size_base.unsigned_abs() - 1) * PERPENDICULAR_SCAN_WIDTH,
                );
                // trace!(
                //     "start moving side {}",
                //     perpendicular_scan_area.unsigned_abs() - 1
                // );
                rail
            };

            if !bounding_area.contains_point(&scan_start.endpoint) {
                break;
            }

            let scan_end = {
                let mut rail = scan_start.clone();
                // We are still perpendicular
                rail = rail.move_forward_step_num(PERPENDICULAR_SCAN_WIDTH);

                if !bounding_area.contains_point(&rail.endpoint) {
                    // went too far
                    break;
                }

                // Go all the way to the edge
                rail.direction = cross_side.direction.clone();
                while bounding_area.contains_point(&rail.endpoint) {
                    rail = rail.move_forward_step();
                }
                // Now outside, go back
                rail.move_force_rotate_clockwise(2);
                rail = rail.move_forward_step();

                // trace!("end moving side {}", perpendicular_scan_area.unsigned_abs());
                rail
            };

            let search_area =
                VArea::from_arbitrary_points_pair(scan_start.endpoint, scan_end.endpoint);
            let found_mines: Vec<MineBase> = mines
                .extract_if(|mine| {
                    // search_area.contains_point(&mine.area.start)
                    //     || search_area.contains_point(&mine.area.point_bottom_left())
                    search_area.contains_point(&mine.area.point_center())
                })
                // TODO: Support multiple directions
                .sorted_by_key(|mine| mine.area.start.x())
                .collect();
            for mine in &found_mines {
                trace!("batch for mine {:?}", mine);
            }

            // TODO: multiple sides
            let delta_y_base = scan_start.endpoint.y().abs_diff(scan_end.endpoint.y()) as i32;
            let delta_y = delta_y_base * 3;
            let batch_search_area = VArea::from_arbitrary_points([
                scan_start.endpoint.move_y(-delta_y),
                scan_start.endpoint.move_y(delta_y),
                scan_end.endpoint.move_y(-delta_y),
                scan_end.endpoint.move_y(delta_y),
                base_source_eighth.lock().unwrap().peek_add(0),
            ]);
            // VPoint::new(
            //     base_source_eighth.lock().unwrap().peek_add(0).x(),
            //     scan_start.endpoint.y(),
            // ),

            batches.push(MineBaseBatch {
                mines: found_mines,
                base_direction: cross_side.direction.clone(),
                base_source_eighth: base_source_eighth.clone(),
                batch_search_area,
            });
            // if 1 + 1 == 2 {
            //     break 'outer;
            // }
        }
    }
    batches
}

fn patches_by_radial_base_corner(surface: &VSurface, resource: Pixel) -> Vec<&VPatch> {
    let patches: Vec<&VPatch> = surface
        .get_patches_slice()
        .iter()
        // remove inner base patches
        .filter(|p| {
            !p.area
                .start
                .is_within_center_radius(REMOVE_RESOURCE_BASE_TILES as u32)
        })
        // temporary left of box only
        .filter(|p| {
            (-REMOVE_RESOURCE_BASE_TILES..REMOVE_RESOURCE_BASE_TILES).contains(&p.area.start.y())
                && p.area.start.x() > REMOVE_RESOURCE_BASE_TILES
        })
        .filter(|v| v.resource == resource)
        .collect();
    let cloud = map_vpatch_to_kdtree(patches.iter());

    let base_corner = base_bottom_right_corner();
    let nearest: Vec<NearestNeighbour<f32, usize>> =
        cloud.nearest_n::<Manhattan>(&base_corner.to_slice_f32(), MAX_PATCHES);
    debug!("found {} from {}", nearest.len(), cloud.size());

    nearest
        .iter()
        .map(|neighbor| patches[neighbor.item])
        .collect()
}

pub fn base_bottom_right_corner() -> VPoint {
    VPoint::new(CENTRAL_BASE_TILES, CENTRAL_BASE_TILES)
}

impl MineBase {
    pub fn get_vpatches<'a>(&self, surface: &'a VSurface) -> Vec<&'a VPatch> {
        self.patch_indexes
            .iter()
            .map(|patch_index| &surface.get_patches_slice()[*patch_index])
            .collect()
    }
}
