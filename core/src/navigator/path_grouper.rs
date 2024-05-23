use crate::navigator::mori::{Rail, RailDirection};
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
use tracing::debug;

const MAX_PATCHES: usize = 200;

pub fn get_patches(surface: &VSurface) -> Vec<&VPatch> {
    let ordered_patches = match 2 {
        1 => patches_by_radial_base_corner(surface, Pixel::IronOre),
        2 => patches_by_cross_sign_expanding(
            surface,
            &[Pixel::IronOre, Pixel::CopperOre, Pixel::Stone, Pixel::Coal],
        ),
        _ => panic!("asd"),
    };
    ordered_patches
}

#[derive(Clone)]
pub struct PatchGroup {
    patch_indexes: Vec<usize>,
    area: VArea,
}

pub fn group_nearby_patches(surface: &VSurface, patches: &[&VPatch]) -> Vec<PatchGroup> {
    let patch_range = 0..patches.len();
    let mut group_ids: Vec<Option<usize>> = patch_range.clone().map(|v| None).collect();

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
        let patch_group;
        if let Some(group_id) = group_ids[source_index] {
            if group_id != source_index {
                // handled first see
                continue;
            }
            patch_group = group_ids
                .iter()
                .enumerate()
                // get all of our group ids, then get the actual patch they point to
                .filter_map(|(search_index, group_id_opt)| {
                    group_id_opt
                        .filter(|search_group_id| *search_group_id == group_id)
                        .map(|_search_group_id| patches[search_index])
                })
                // unwrap Some to get our patches
                .collect()
        } else {
            patch_group = vec![patches[source_index]];
        }

        // Externally the index in the VSurface Patches slice is used
        let patch_group_indexes = patch_group
            .iter()
            .map(|patch| {
                surface
                    .get_patches_slice()
                    .iter()
                    .find_position(|surface_patch| **patch == **surface_patch)
                    .unwrap()
                    .0
            })
            .collect();

        let patch_group_points: Vec<VPoint> = patch_group
            .iter()
            .flat_map(|patch| [patch.area.point_bottom_left(), patch.area.start])
            .collect();

        result.push(PatchGroup {
            patch_indexes: patch_group_indexes,
            area: VArea::from_arbitrary_points(&patch_group_points),
        })
    }

    result
}

// #[allow(clippy::never_loop)]
fn patches_by_cross_sign_expanding<'a>(
    surface: &'a VSurface,
    resources: &[Pixel],
) -> Vec<&'a VPatch> {
    const PERPENDICULAR_SCAN_WIDTH: i32 = 10;

    let cross_sides = [
        // Rail::new_straight(
        //     VPoint::new(REMOVE_RESOURCE_BASE_TILES, 0),
        //     RailDirection::Right,
        // )
        Rail::new_straight(
            VPoint::new(REMOVE_RESOURCE_BASE_TILES, 0),
            RailDirection::Right,
        ),
    ];
    let mut patches = Vec::new();
    for cross_side in cross_sides {
        for perpendicular_scan_area in (1i32..).flat_map(|i| [i, -i]) {
            let perpendicular_scan_area = perpendicular_scan_area * PERPENDICULAR_SCAN_WIDTH;
            // if perpendicular_scan_area.unsigned_abs() * RAIL_STEP_SIZE > surface.get_radius() {
            //     break;
            // }

            let mut parallel_scan_area = 0;
            loop {
                parallel_scan_area += 1;

                // let scan_start = {
                //     let mut rail = cross_side.clone();
                //     for _ in 0..(parallel_scan_area - 1) {
                //         rail = rail.move_forward_step();
                //     }
                //     rail
                // };

                let scan_start = {
                    let mut rail = cross_side.move_forward_step_num(parallel_scan_area - 1);
                    // trace!("start moving forward {}", parallel_scan_area - 1);
                    rail = if perpendicular_scan_area > 0 {
                        rail.move_force_rotate_clockwise(1)
                    } else {
                        rail.move_force_rotate_clockwise(3)
                    };
                    rail = rail.move_forward_step_num(perpendicular_scan_area.unsigned_abs() - 1);
                    // trace!(
                    //     "start moving side {}",
                    //     perpendicular_scan_area.unsigned_abs() - 1
                    // );
                    rail
                };

                let scan_end = {
                    let mut rail = cross_side.move_forward_step_num(parallel_scan_area);
                    // trace!("end moving forward {}", parallel_scan_area);
                    rail = if perpendicular_scan_area > 0 {
                        rail.move_force_rotate_clockwise(1)
                    } else {
                        rail.move_force_rotate_clockwise(3)
                    };
                    rail = rail.move_forward_step_num(perpendicular_scan_area.unsigned_abs());
                    // trace!("end moving side {}", perpendicular_scan_area.unsigned_abs());
                    rail
                };

                if !scan_end
                    .endpoint
                    .is_within_center_radius(surface.get_radius())
                {
                    break;
                }

                let search_area =
                    VArea::from_arbitrary_points_pair(&scan_start.endpoint, &scan_end.endpoint);
                for patch in surface.get_patches_slice() {
                    if resources.contains(&patch.resource)
                        && search_area.contains_point(&patch.area.start)
                    {
                        patches.push(patch);
                    }
                }
            }
        }
    }
    patches
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
