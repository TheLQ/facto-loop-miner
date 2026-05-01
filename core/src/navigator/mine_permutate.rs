use crate::navigator::base_source::BaseSourceEighth;
use crate::navigator::mine_executor::{ExecutionRoute, ExecutionSequence};
use crate::navigator::mine_selector::MineSelectBatch;
use crate::surfacev::mine::{MineDestination, MineLocation};
use crate::surfacev::vsurface::{
    MineDestinationRef, MineRef, VSurfaceMine, VSurfaceMineAsVs, VSurfacePatch, VSurfacePixel,
};
use facto_loop_miner_fac_engine::common::varea::VArea;
use facto_loop_miner_fac_engine::common::vpoint::{VPOINT_ZERO, VPoint};
use facto_loop_miner_fac_engine::common::vpoint_direction::VPointDirectionQ;
use facto_loop_miner_fac_engine::game_blocks::rail_hope_single::SECTION_POINTS_I32;
use itertools::Itertools;
use tracing::warn;

/// Input
///  - Single batch of mines to be routed together
///
/// Output
///  - Each mine has 4 destinations
///  - Therefore batch has 4^n possible combinations
///  - Combinations each can be permutated generating n! combinations
pub fn get_possible_routes_for_batch<'plan_mine>(
    surface: VSurfaceMine,
    MineSelectBatch { mines }: MineSelectBatch,
    fixed_finding_limiter: VArea,
) -> CompletePlan {
    let mines_len = mines.len();
    // let mines_destinations_len: usize = mines.iter().map(|v| v.destinations().len()).sum();
    // info!(
    //     "Expanded {} mines with {} destinations to...",
    //     mines_len, mines_destinations_len,
    // );
    assert!(!mines.is_empty(), "nope");

    let resolved_mines = surface.resolve_mines_with_refs(mines).collect_vec();

    let mine_combinations = find_all_combinations(&resolved_mines);
    assert!(!mine_combinations.is_empty(), "nope");
    // let total_combinations_base = mine_combinations.len();
    let mine_combinations = find_all_permutations(mine_combinations);
    // let total_combinations_permut = mine_combinations.len();

    // info!(
    //     "Expanded {} mines with {} destinations to {} combinations then {} permutated",
    //     mines_len,
    //     mines_destinations_len,
    //     total_combinations_base,
    //     total_combinations_permut
    // );

    // Did we actually generate unique steps?
    // let mut dedupe_test = mine_combinations.iter().collect_vec();
    // dedupe_test.sort();
    // dedupe_test.dedup();
    // let dedupe_len = dedupe_test.len();
    // assert_eq!(total_combinations_permut, dedupe_len);

    let sequences = build_routes_from_destinations(mine_combinations, fixed_finding_limiter);
    // assert!(
    //     !sequences.is_empty(),
    //     "no sequences found from {mines_len} input mines"
    // );
    if sequences.is_empty() {
        warn!("no sequences found from {mines_len} input mines");
    }
    CompletePlan { sequences }
}

pub struct CompletePlan {
    pub sequences: Vec<ExecutionSequence>,
}

/// Find all combinations of `a[1,2,3,4], b[1,2,3,4], ... = [a1, b1], [a2, b2], ...`
/// This is <4^n sized Vec, because of the 4 possible choices.
///
/// Start with a list of mines with 4x possible positions.
/// Create combinations of `[a1, b1, c2, ...]`
fn find_all_combinations(mines: &[(MineRef, &MineLocation)]) -> Vec<Vec<MineDestinationRef>> {
    fn recurse(
        path: Vec<MineDestinationRef>,
        remain: &[(MineRef, &MineLocation)],
        output: &mut Vec<Vec<MineDestinationRef>>,
    ) {
        if let Some((mine_ref, mine)) = remain.first() {
            for (dest_ref, destination) in mine.destinations_with_refs(*mine_ref) {
                let mut next_path = path.clone();
                next_path.push(dest_ref);
                recurse(next_path, &remain[1..], output);
            }
        } else {
            output.push(path);
        }
    }

    let mut routes: Vec<Vec<MineDestinationRef>> = Vec::new();
    recurse(Vec::new(), mines, &mut routes);
    routes
}

/// Find all re-ordered permutations of `[a,b,c,...] = n!`
/// This is huge
fn find_all_permutations(
    input_combinations: Vec<Vec<MineDestinationRef>>,
) -> Vec<Vec<MineDestinationRef>> {
    input_combinations
        .into_iter()
        .flat_map(|combination| {
            let total_combinations = combination.len();
            combination.into_iter().permutations(total_combinations)
        })
        .collect()
}

/// Add the base source rail going to the destination, in order
fn build_routes_from_destinations(
    input_combinations: Vec<Vec<MineDestinationRef>>,
    fixed_finding_limiter: VArea,
) -> Vec<ExecutionSequence> {
    let mut sequences: Vec<ExecutionSequence> = Vec::new();
    'combinations: for combination in input_combinations {
        let mut sequence: Vec<ExecutionRoute> = Vec::new();
        for (i, destination) in combination.into_iter().enumerate() {
            sequence.push(ExecutionRoute {
                destination,
                finding_limiter: fixed_finding_limiter.clone(),
            })
        }
        sequences.push(ExecutionSequence::new(sequence));
    }
    sequences
}
