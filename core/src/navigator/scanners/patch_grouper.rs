use crate::TILES_PER_CHUNK;
use crate::surface::pixel::Pixel;
use crate::surfacev::vsurface::{PatchRef, VSurfacePatch};
use itertools::Itertools;

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
        if processed_patches.contains(patch_i) {
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

fn recursive_near_patches(
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
