use crate::navigator::mori::{Rail, RailDirection, RailMode};
use crate::navigator::path_side::{BaseSource, BaseSourceEighth};
use crate::state::machine_v1::{CENTRAL_BASE_TILES, REMOVE_RESOURCE_BASE_TILES};
use crate::surface::patch::map_vpatch_to_kdtree;
use crate::surface::pixel::Pixel;
use crate::surfacev::varea::VArea;
use crate::surfacev::vpatch::VPatch;
use crate::surfacev::vpoint::VPoint;
use crate::surfacev::vsurface::VSurface;
use crate::TILES_PER_CHUNK;
use itertools::Itertools;
use kiddo::{Manhattan, NearestNeighbour};
use std::rc::Rc;
use tracing::{debug, trace};

const MAX_PATCHES: usize = 200;

#[derive(Clone, Debug)]
pub struct MineBase {
    pub patch_indexes: Vec<usize>,
    pub area: VArea,
}

pub struct MineBaseBatch {
    pub mines: Vec<MineBase>,
    pub base_source_eighth: Rc<BaseSourceEighth>,
    pub base_direction: RailDirection,
}

/// Solve these core problems
/// - Find the patches we care about
/// - Create batches that can be optimized as a group, because larger groups are exponentially more difficult to optimize
pub fn get_mine_bases_by_batch(surface: &VSurface, base_source: &BaseSource) -> Vec<MineBaseBatch> {
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
    patches_by_cross_sign_expanding(patch_groups, base_source)
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
    let mut group_ids: Vec<Option<usize>> = patch_range.clone().map(|v| None).collect();

    // find nearest paths
    // brute force alternative algo to kdtree for fun
    for source_index in patch_range.clone() {
        if group_ids[source_index].is_some() {
            continue;
        }
        for other_index in patch_range.clone() {
            if source_index == other_index || group_ids[other_index].is_some() {
                continue;
            }
            let source_patch = patches[source_index];
            let other_patch = patches[other_index];

            if source_patch
                .area
                .point_center()
                .distance_bird(&other_patch.area.point_center())
                < TILES_PER_CHUNK as f32 / 2.0
            {
                group_ids[source_index] = Some(source_index);
                group_ids[other_index] = Some(source_index);
            }
        }
    }

    // combine like ids
    let mut result = Vec::new();

    for source_index in patch_range {
        if let Some(group_id) = group_ids[source_index] {
            if group_id != source_index {
                // already handled first see
                continue;
            }
            let patch_group: Vec<&VPatch> = group_ids
                .iter()
                .enumerate()
                // get all of our group ids, then get the actual patch they point to
                .filter_map(|(search_index, group_id_opt)| {
                    group_id_opt
                        .filter(|search_group_id| *search_group_id == group_id)
                        .map(|_search_group_id| patches[search_index])
                })
                .collect();
            trace!("Merging patch group of {:?}", patch_group);

            // Externally we use the index in the VSurface Patches slice
            let patch_group_indexes = patch_group
                .iter()
                .map(|patch| patch.get_surface_patch_index(surface))
                .collect();

            let patch_group_points: Vec<VPoint> = patch_group
                .iter()
                .flat_map(|patch| [patch.area.point_bottom_left(), patch.area.start])
                .collect();

            result.push(MineBase {
                patch_indexes: patch_group_indexes,
                area: VArea::from_arbitrary_points(&patch_group_points),
            });
            // panic!("TODO: Broken area");
        } else {
            let patch = patches[source_index];
            trace!("Single patch group {:?}", patch);
            result.push(MineBase {
                patch_indexes: vec![patch.get_surface_patch_index(surface)],
                area: patch.area.clone(),
            })
        }
    }

    result
}

// #[allow(clippy::never_loop)]
fn patches_by_cross_sign_expanding(
    mut mines: Vec<MineBase>,
    base_source: &BaseSource,
) -> Vec<MineBaseBatch> {
    const PERPENDICULAR_SCAN_WIDTH: u32 = 10;

    let bounding_area = VArea::from_arbitrary_points(
        &mines
            .iter()
            .flat_map(|v| v.area.get_corner_points())
            .collect_vec(),
    );
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
    'outer: for cross_side in cross_sides {
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
                VArea::from_arbitrary_points_pair(&scan_start.endpoint, &scan_end.endpoint);
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
            batches.push(MineBaseBatch {
                mines: found_mines,
                base_direction: cross_side.direction.clone(),
                base_source_eighth: base_source_eighth.clone(),
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
