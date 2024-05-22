use crate::navigator::mori::Rail;
use crate::navigator::path_grouper::PatchGroup;
use crate::surfacev::vsurface::VSurface;
use itertools::Itertools;

/// Solve 2 core problems
/// - Get an ordered list of patches to navigate to
/// - (!) Provide multiple possible solutions when, due to patching problems,
///   - We cannot reach a patch anymore and must discard it
///   - Another corner creates a more optimal/lower-cost path
///   - Different order creates a more optimal/lower-cost path
///
/// Total paths = N *
pub fn get_patches_to_drive(surface: &VSurface) -> Vec<Vec<PatchPlan>> {
    let result = Vec::new();
    // info!("Getting for {} entries", self.entries.len());
    result
}

const DEST_SIZE: usize = 4;

struct PatchPlanAnyDestinations {
    patch_group: PatchGroup,
    destinations: [Rail; DEST_SIZE],
}

#[derive(Clone)]
pub struct PatchPlan {
    patch_group: PatchGroup,
    entry_rail: Rail,
}

/// Find all combinations of `a[1,2,3,4], b[1,2,3,4], ... = 4^n` sized Vec.
/// This is huge.
fn find_all_combinations(plans: &[PatchPlanAnyDestinations]) -> Vec<Vec<PatchPlan>> {
    let mut result = Vec::new();
    for plan in plans {
        if result.is_empty() {
            // seed
            for outpost in plan.to_patch_outposts() {
                result.push(vec![outpost]);
            }
        } else {
            // every existing entry is cloned 4x for the new destinations
            let mut new_result = Vec::new();
            for entry in result {
                for outpost in plan.to_patch_outposts() {
                    let mut new_entry = entry.clone();
                    new_entry.push(outpost);
                    new_result.push(new_entry);
                }
            }
            result = new_result;
        }
    }
    result
}

/// Find all permutations of `[a,b,c,...] = n!`
fn find_all_permutations(input: Vec<Vec<PatchPlan>>) -> Vec<Vec<PatchPlan>> {
    let input_len = input.len();
    let mut result: Vec<Vec<PatchPlan>> = Vec::new();
    for entry in input {
        for permutation in entry.iter().permutations(input_len) {
            result.push(permutation.into_iter().cloned().collect());
        }
    }
    result
}

impl PatchPlanAnyDestinations {
    // fn to_patch_outpost(&self, destination_index: usize) -> PatchOutpost {
    //     PatchOutpost {
    //         patch_indexes: self.patch_indexes.clone(),
    //         area: self.area.clone(),
    //         entry_rail: self.destinations[destination_index].clone()
    //     }
    // }

    fn to_patch_outposts(&self) -> [PatchPlan; DEST_SIZE] {
        self.destinations.clone().map(|destination| PatchPlan {
            patch_group: self.patch_group.clone(),
            entry_rail: destination.clone(),
        })
    }
}

impl PatchPlan {
    // fn from_patch_block(block: &PatchOutpostAll, index: usize) -> Self {
    //     PatchOutpost {
    //         patch_indexes: block.patch_indexes.clone(),
    //         area: block.area.clone(),
    //         entry_rail: block.destinations[index].clone(),
    //     }
    // }
}
