use crate::navigator::path_side::BaseSource;
use crate::state::machine_v1::REMOVE_RESOURCE_BASE_TILES;
use crate::surface::pixel::Pixel;
use crate::surfacev::mine::MineLocation;
use crate::surfacev::vpatch::VPatch;
use crate::surfacev::vsurface::VSurface;
use crate::TILES_PER_CHUNK;
use facto_loop_miner_fac_engine::common::varea::VArea;
use facto_loop_miner_fac_engine::common::vpoint::{VPoint, VPOINT_TEN};
use facto_loop_miner_fac_engine::common::vpoint_direction::VPointDirectionQ;
use facto_loop_miner_fac_engine::game_entities::direction::FacDirectionQuarter;
use itertools::Itertools;
use tracing::{debug, error, warn};

const MAX_PATCHES: usize = 200;

pub struct MineSelectBatch {
    pub mines: Vec<MineLocation>,
    pub base_sources: Vec<VPointDirectionQ>,
}

pub enum MineSelectBatchResult {
    Success { batches: Vec<MineSelectBatch> },
    EmptyBatch,
}

impl MineSelectBatchResult {
    pub fn into_success(self) -> Option<Vec<MineSelectBatch>> {
        match self {
            MineSelectBatchResult::Success { batches } => Some(batches),
            MineSelectBatchResult::EmptyBatch { .. } => None,
        }
    }
}

const MAXIMUM_MINE_COUNT_PER_BATCH: usize = 5;
/// at 3000 crop
/// - 20 generates mostly 1, 2, some 3
/// - 40 generates slightly more 3
/// - 80 generates way less 1, more 2, good 3,4
/// - 160 generates mostly 3 - very good
/// - 220 generates 10 batch, too big
const PERPENDICULAR_SCAN_WIDTH: i32 = 120;

/// Input:
///  - Raw patch list
/// Output:
///  - Group nearby patches
///  - Order patch groups starting from center
///  - Assign base sources
///  - Split groups if needed because too huge creates too many possibilities later
pub fn select_mines_and_sources(surface: &VSurface) -> MineSelectBatchResult {
    let base_source = &mut BaseSource::new(VPointDirectionQ(VPOINT_TEN, FacDirectionQuarter::East));

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
    if mine_batches.is_empty() {
        return MineSelectBatchResult::EmptyBatch;
    }

    let mut result = Vec::new();
    for (index, mine_batch) in mine_batches.into_iter().enumerate() {
        // When expanded, 6! = 720. 9! = 362,880 which is too gigantic

        let batch_mines_len = mine_batch.mines.len();
        if mine_batch.mines.is_empty() {
            error!("bad batch at {}", index);
        } else if batch_mines_len > MAXIMUM_MINE_COUNT_PER_BATCH {
            let mut divisor = 2;
            while batch_mines_len / divisor > MAXIMUM_MINE_COUNT_PER_BATCH {
                divisor += 1;
                warn!("increasing divisor to {divisor} total {batch_mines_len}")
            }
            let chunk_size = batch_mines_len / divisor;
            debug!("index {index} split {batch_mines_len} by {divisor}");

            let mut base_sources = mine_batch.base_sources;
            for chunk in &mine_batch.mines.into_iter().chunks(chunk_size) {
                let mines: Vec<MineLocation> = chunk.into_iter().collect();
                let base_sources = base_sources.drain(0..mines.len()).collect();
                result.push(MineSelectBatch {
                    mines,
                    base_sources,
                });
            }
        } else {
            result.push(mine_batch);
        }
    }
    MineSelectBatchResult::Success { batches: result }
}

/// Second grouping pass (after opencv), now by grouping different resource patches
fn group_nearby_patches(surface: &VSurface, resources: &[Pixel]) -> Vec<MineLocation> {
    let patches: Vec<&VPatch> = surface
        .get_patches_slice()
        .iter()
        .filter(|patch| resources.contains(&patch.resource))
        .collect();
    let patch_range = 0..patches.len();

    // group patches by nearby
    let mut groups: Vec<Vec<&VPatch>> = Vec::new();
    for patch in &patches {
        // todo: this was a performance boost. Needs to re-benchmark
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
            // trace!("Merging patch group of {:?}", patch_group);

            // Externally we use the index in the VSurface Patches slice
            let patch_group_indexes = patch_group
                .iter()
                .map(|patch| patch.get_surface_patch_index(surface))
                .collect();

            let area = VArea::from_arbitrary_points(
                patch_group.iter().flat_map(|patch| &patch.pixel_indexes),
            );

            result.push(MineLocation {
                patch_indexes: patch_group_indexes,
                area,
            });
        } else {
            let patch = patch_group[0];
            // trace!("Single patch group {:?}", patch);
            result.push(MineLocation {
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

fn patches_by_cross_sign_expanding(
    mut mines: Vec<MineLocation>,
    base_source: &mut BaseSource,
) -> Vec<MineSelectBatch> {
    let bounding_area =
        VArea::from_arbitrary_points(mines.iter().flat_map(|v| v.area.get_corner_points()));
    let cross_sides: [VPointDirectionQ; 1] = [
        // Rail::new_straight(
        //     VPoint::new(REMOVE_RESOURCE_BASE_TILES, 0),
        //     RailDirection::Right,
        // )
        VPointDirectionQ(
            VPoint::new(REMOVE_RESOURCE_BASE_TILES, 0),
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
                    scan_index * PERPENDICULAR_SCAN_WIDTH,
                );
            if !bounding_area.contains_point(&scan_start) {
                // extended past edge of surface
                break;
            }

            let scan_end = {
                let mut pos = scan_start;
                // move up again to complete box height
                // this is the only way to be generic. not a hot path though
                for _ in 0..PERPENDICULAR_SCAN_WIDTH {
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
                .extract_if(0..mines.len(), |mine| {
                    search_area.contains_point(&mine.area.start)
                        || search_area.contains_point(&mine.area.point_bottom_right())
                })
                .collect();
            if found_mines.is_empty() {
                // might just be unlucky with small scan areas
                continue;
            }
            found_mines.sort_by(|left, right| {
                VPoint::sort_by_direction(
                    *cross_side.direction(),
                    left.area.start,
                    right.area.start,
                )
            });
            // for mine in &found_mines {
            //     trace!("batch for mine {:?}", mine);
            // }

            // TODO: Support multiple sides
            let base_source_eighth = if scan_index > 0 {
                // TODO: While going positive we get a... negative position?
                base_source.negative()
            } else {
                base_source.positive()
            };

            let found_mines_len = found_mines.len();
            batches.push(MineSelectBatch {
                mines: found_mines,
                base_sources: base_source_eighth.take(found_mines_len).collect(),
            });
        }
    }
    batches
}

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
