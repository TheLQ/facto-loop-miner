use crate::TILES_PER_CHUNK;
use crate::navigator::base_source::BaseSourceEighth;
use crate::navigator::planners::PathingTunables;
use crate::surface::pixel::Pixel;
use crate::surfacev::iter_remain_util::RemainIter;
use crate::surfacev::mine::MineLocation;
use crate::surfacev::vsurface::{MineRef, PatchRef, VSurfacePatch};
use itertools::Itertools;
use simd_json::prelude::ArrayTrait;

pub struct MineSelectBatch<'plan_mine> {
    pub mines: Vec<(MineRef, &'plan_mine MineLocation)>,
}

pub enum MineSelectBatchResult<'plan_mine> {
    Success {
        batches: Vec<MineSelectBatch<'plan_mine>>,
    },
    EmptyBatch,
}

impl<'plan_mine> MineSelectBatchResult<'plan_mine> {
    pub fn into_success(self) -> Option<Vec<MineSelectBatch<'plan_mine>>> {
        match self {
            MineSelectBatchResult::Success { batches } => Some(batches),
            MineSelectBatchResult::EmptyBatch => None,
        }
    }
}

/// Input:
///  - Raw patch list
///
/// Output:
///  - Group nearby patches
///  - Order patch groups starting from center
///  - Assign base sources
///  - Split groups if needed because too huge creates too many possibilities later
pub fn select_mines_and_sources<'plan_mine>(
    _tunables: &PathingTunables,
    _surface: VSurfacePatch,
    _maximum_mine_count_per_batch: usize,
) -> MineSelectBatchResult<'plan_mine> {
    todo!()
    /*
    let base_source = BaseSource::from_central_base(tunables).into_positive();

    let patch_groups = group_nearby_patches(surface);
    let total_patches: usize = patch_groups.iter().map(|v| v.len()).sum();
    info!("selected {total_patches} patches");

    // let ordered_patches = match 2 {
    //     1 => patches_by_radial_base_corner(surface, Pixel::IronOre),
    //     // 2 => patches_by_cross_sign_expanding(
    //     //     surface,
    //     //     &[Pixel::IronOre, Pixel::CopperOre, Pixel::Stone, Pixel::Coal],
    //     // ),
    //     _ => panic!("asd"),
    // };
    // ordered_patches

    let mine_batches =
        patches_by_cross_sign_expanding(/*patch_groups*/ todo!(), base_source, tunables);
    if mine_batches.is_empty() {
        return MineSelectBatchResult::EmptyBatch;
    }

    let mut result = Vec::new();
    for (index, mine_batch) in mine_batches.into_iter().enumerate() {
        // When expanded, 6! = 720. 9! = 362,880 which is too gigantic

        let batch_mines_len = mine_batch.mines.len();
        if mine_batch.mines.is_empty() {
            error!("bad batch at {}", index);
        } else if batch_mines_len > maximum_mine_count_per_batch {
            let mut divisor = 2;
            while batch_mines_len / divisor > maximum_mine_count_per_batch {
                divisor += 1;
                warn!("increasing divisor to {divisor} total {batch_mines_len}")
            }
            let chunk_size = batch_mines_len / divisor;
            debug!("index {index} split {batch_mines_len} by {divisor}");

            for chunk in &mine_batch.mines.into_iter().chunks(chunk_size) {
                let mines: Vec<MineLocation> = chunk.into_iter().collect();
                result.push(MineSelectBatch {
                    mines,
                    base_sources: mine_batch.base_sources.clone(),
                });
            }
        } else {
            result.push(mine_batch);
        }
    }
    MineSelectBatchResult::Success { batches: result }
     */
}

/// * First, opencv groups raw pixels into per-resource patches
/// * Second, group any patches nearby each-other
pub fn group_nearby_patches(surface: VSurfacePatch) -> Vec<Vec<PatchRef>> {
    // ignores UraniumOre because it's only for
    // electric production (solar instead) and military (unused)
    let resources = [
        Pixel::IronOre,
        Pixel::CopperOre,
        Pixel::Stone,
        Pixel::Coal,
        Pixel::CrudeOil,
    ];

    let all_patches: Vec<PatchRef> = surface
        .patches_with_index()
        .filter(|(_, patch)| resources.contains(&patch.resource))
        .map(|(i, _)| i)
        .collect();
    let mut processed_patches: Vec<PatchRef> = Vec::new();

    let mut groups: Vec<Vec<PatchRef>> = Vec::new();
    for patch_i in &all_patches {
        if processed_patches.contains(&patch_i) {
            // already in a group
            continue;
        }

        let mut new_group = Vec::new();
        new_group.push(patch_i.clone());
        recursive_near_patches(patch_i, &all_patches, &mut new_group, surface);
        for patch_j in &new_group {
            processed_patches.push(patch_j.clone());
        }

        groups.push(new_group);
    }

    {
        let mut dedupe_check = groups.iter().flatten().cloned().collect_vec();
        let old = dedupe_check.len();
        dedupe_check.sort();
        dedupe_check.dedup();
        assert_eq!(old, dedupe_check.len(), "dedupe found stuff!");
    }

    groups
}

fn recursive_near_patches<'a>(
    needle: &PatchRef,
    remaining_patches: &[PatchRef],
    result: &mut Vec<PatchRef>,
    surface: VSurfacePatch,
) {
    for other in remaining_patches {
        // assert_ne!(other, needle);
        if other == needle || result.contains(other) {
            continue;
        }

        let needle_patch = needle.get_patch(surface);
        let other_patch = other.get_patch(surface);

        if needle_patch
            .area
            .point_center()
            .distance_bird(&other_patch.area.point_center())
            < TILES_PER_CHUNK as f32 * 3.0
        {
            result.push((*other).clone());
            // recursive_near_patches(other, &remaining_patches[1..], result, surface);
            recursive_near_patches(other, remaining_patches, result, surface);
        }
    }
}

/*
fn patches_by_cross_sign_expanding(
    mut mines: Vec<MineLocation>,
    base_sources: BaseSourceEighth,
    base_tunables: &PathingTunables,
) -> Vec<MineSelectBatch> {
    let bounding_area =
        VArea::from_arbitrary_points(mines.iter().flat_map(|v| v.area_min().get_corner_points()));

    let cross_sides: [VPointDirectionQ; 1] = [
        // Rail::new_straight(
        //     VPoint::new(REMOVE_RESOURCE_BASE_TILES, 0),
        //     RailDirection::Right,
        // )
        VPointDirectionQ(
            // todo: this assumes dream of both east and west building
            VPoint::new(base_tunables.base_chunks().as_tiles_i32(), 0),
            FacDirectionQuarter::East,
        ),
    ];
    let mut batches = Vec::new();
    for cross_side in cross_sides {
        for scan_index in (1i32..).flat_map(|i| [i, -i]) {
            let scan_start = cross_side
                .point()
                // first corner
                .move_direction_sideways_int(
                    cross_side.direction(),
                    scan_index * base_tunables.path_common().scan_size,
                );
            if !bounding_area.contains_point(&scan_start) {
                // extended past edge of surface
                break;
            }

            let scan_end = {
                let mut pos = scan_start;
                // move up again to complete box height
                // this is the only way to be generic. not a hot path though
                for _ in 0..base_tunables.path_common().scan_size {
                    let next = pos.move_direction_sideways_int(cross_side.direction(), 1);
                    if bounding_area.contains_point(&next) {
                        pos = next;
                    } else {
                        break;
                    }
                }

                // move left to edge of surface
                // again trying to be generic
                loop {
                    let next = pos.move_direction_int(cross_side.direction(), 1);
                    if bounding_area.contains_point(&next) {
                        pos = next;
                    } else {
                        break;
                    }
                }
                pos
            };

            let search_area = VArea::from_arbitrary_points_pair(scan_start, scan_end);
            let mut found_mines: Vec<MineLocation> = mines
                .extract_if(.., |mine| {
                    search_area.contains_point(&mine.area_min().point_center())
                })
                .collect();
            if found_mines.is_empty() {
                // might just be unlucky with small scan areas
                continue;
            }
            found_mines.sort_by(|left, right| {
                VPoint::sort_by_direction(
                    *cross_side.direction(),
                    left.area_min().point_top_left(),
                    right.area_min().point_top_left(),
                )
            });
            // for mine in &found_mines {
            //     trace!("batch for mine {:?}", mine);
            // }

            batches.push(MineSelectBatch {
                mines: found_mines,
                base_sources: base_sources.clone(),
            });
        }
    }
    batches
}
*/

// fn patches_by_radial_base_corner(surface: &VSurface, resource: Pixel) -> Vec<&VPatch> {
//     let patches: Vec<&VPatch> = surface
//         .get_patches_slice()
//         .iter()
//         // remove inner base patches
//         .filter(|p| {
//             !p.area
//                 .start
//                 .is_within_center_radius(REMOVE_RESOURCE_BASE_TILES as u32)
//         })
//         // temporary left of box only
//         .filter(|p| {
//             (-REMOVE_RESOURCE_BASE_TILES..REMOVE_RESOURCE_BASE_TILES).contains(&p.area.start.y())
//                 && p.area.start.x() > REMOVE_RESOURCE_BASE_TILES
//         })
//         .filter(|v| v.resource == resource)
//         .collect();
//     let cloud = map_vpatch_to_kdtree(patches.iter());
//
//     let base_corner = base_bottom_right_corner();
//     let nearest: Vec<NearestNeighbour<f32, usize>> =
//         cloud.nearest_n::<Manhattan>(&base_corner.to_slice_f32(), MAX_PATCHES);
//     debug!("found {} from {}", nearest.len(), cloud.size());
//
//     nearest
//         .iter()
//         .map(|neighbor| patches[neighbor.item])
//         .collect()
// }
