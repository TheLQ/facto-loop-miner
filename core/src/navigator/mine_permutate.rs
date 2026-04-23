use crate::navigator::base_source::BaseSourceEighth;
use crate::navigator::mine_executor::{ExecutionRoute, ExecutionSequence};
use crate::navigator::mine_selector::MineSelectBatch;
use crate::surfacev::mine::{MineDestination, MineLocation};
use crate::surfacev::vsurface::VSurfacePixel;
use facto_loop_miner_fac_engine::common::varea::VArea;
use facto_loop_miner_fac_engine::common::vpoint::VPoint;
use facto_loop_miner_fac_engine::common::vpoint_direction::VPointDirectionQ;
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
    surface: VSurfacePixel,
    MineSelectBatch {
        mines,
        base_sources,
    }: MineSelectBatch<'plan_mine>,
) -> CompletePlan<'plan_mine> {
    let mines_len = mines.len();
    // let mines_destinations_len: usize = mines.iter().map(|v| v.destinations().len()).sum();
    // info!(
    //     "Expanded {} mines with {} destinations to...",
    //     mines_len, mines_destinations_len,
    // );
    assert!(!mines.is_empty(), "nope");

    let mine_combinations = find_all_combinations(mines, &base_sources);
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

    // Limit pathing to the entire right half of the map
    // todo: autogen this somewhere
    let fixed_radius = surface.get_radius_i32();
    let fixed_finding_limiter = VArea::from_arbitrary_points_pair(
        VPoint::new(0, -fixed_radius),
        // Must give spacing from Edge, because hope_link.area() can extend past it.
        // range checks are disabled for theoretical performance
        VPoint::new(fixed_radius, fixed_radius),
    );

    // Did we actually generate unique steps?
    // let mut dedupe_test = mine_combinations.iter().collect_vec();
    // dedupe_test.sort();
    // dedupe_test.dedup();
    // let dedupe_len = dedupe_test.len();
    // assert_eq!(total_combinations_permut, dedupe_len);

    let sequences =
        build_routes_from_destinations(mine_combinations, fixed_finding_limiter, &base_sources);
    // assert!(
    //     !sequences.is_empty(),
    //     "no sequences found from {mines_len} input mines"
    // );
    if sequences.is_empty() {
        warn!("no sequences found from {mines_len} input mines");
    }
    CompletePlan {
        sequences,
        base_sources,
    }
}

pub struct CompletePlan<'plan_mine> {
    pub sequences: Vec<ExecutionSequence<'plan_mine>>,
    pub base_sources: BaseSourceEighth,
}

#[derive(PartialOrd, Ord, PartialEq, Eq, Clone)]
struct PartialEntry<'plan_mine> {
    location: &'plan_mine MineLocation,
    destination: &'plan_mine MineDestination,
}

/// Find all combinations of `a[1,2,3,4], b[1,2,3,4], ... = [a1, b1], [a2, b2], ...`
/// This is <4^n sized Vec, because of the 4 possible choices.
///
/// Start with a list of mines with 4x possible positions.
/// Create combinations of `[a1, b1, c2, ...]`
fn find_all_combinations<'plan_mine>(
    mines: Vec<&'plan_mine MineLocation>,
    base_source: &BaseSourceEighth,
) -> Vec<Vec<PartialEntry<'plan_mine>>> {
    fn recurse<'m>(
        path: Vec<PartialEntry<'m>>,
        remain: &[&'m MineLocation],
        output: &mut Vec<Vec<PartialEntry<'m>>>,
        base_source: &BaseSourceEighth,
    ) {
        if let Some(mine) = remain.first() {
            for destination in mine.destinations() {
                let mut next_path = path.clone();
                next_path.push(PartialEntry {
                    destination,
                    location: mine,
                });
                recurse(next_path, &remain[1..], output, base_source);
            }
        } else {
            output.push(path);
        }
    }

    let mut routes: Vec<Vec<PartialEntry>> = Vec::new();
    recurse(Vec::new(), &mines, &mut routes, base_source);
    routes
}

/// Find all re-ordered permutations of `[a,b,c,...] = n!`
/// This is huge
fn find_all_permutations(input_combinations: Vec<Vec<PartialEntry>>) -> Vec<Vec<PartialEntry>> {
    input_combinations
        .into_iter()
        .flat_map(|combination| {
            let total_combinations = combination.len();
            combination.into_iter().permutations(total_combinations)
        })
        .collect()
}

/// Add the base source rail going to the destination, in order
fn build_routes_from_destinations<'plan_mine>(
    input_combinations: Vec<Vec<PartialEntry<'plan_mine>>>,
    fixed_finding_limiter: VArea,
    base_source: &BaseSourceEighth,
) -> Vec<ExecutionSequence<'plan_mine>> {
    let mut sequences: Vec<ExecutionSequence> = Vec::new();
    'combinations: for combination in input_combinations {
        let mut sequence: Vec<ExecutionRoute> = Vec::new();
        for (
            i,
            PartialEntry {
                destination,
                location,
            },
        ) in combination.into_iter().enumerate()
        {
            let source = base_source.peek_after(i);
            let segment = source.segment_for_mine(destination.for_level(&source.applied_intra));
            if !segment.is_within_area(&fixed_finding_limiter) {
                warn!("segment out of bounds {}", segment);
                continue 'combinations;
            }
            sequence.push(ExecutionRoute {
                segment,
                location,
                finding_limiter: fixed_finding_limiter.clone(),
                intra_level: source.applied_intra,
            })
        }
        sequences.push(ExecutionSequence::new(sequence));
    }
    sequences
}
