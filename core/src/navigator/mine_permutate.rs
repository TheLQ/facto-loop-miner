use crate::navigator::base_source::BaseSourceEighth;
use crate::navigator::mine_executor::{ExecutionRoute, ExecutionSequence};
use crate::navigator::mine_selector::MineSelectBatch;
use crate::surfacev::mine::MineLocation;
use crate::surfacev::vsurface::VSurface;
use facto_loop_miner_fac_engine::common::varea::VArea;
use facto_loop_miner_fac_engine::common::vpoint::VPoint;
use facto_loop_miner_fac_engine::common::vpoint_direction::VPointDirectionQ;
use facto_loop_miner_fac_engine::constants::TILES_PER_CHUNK;
use itertools::Itertools;
use std::cell::RefCell;
use std::ops::DerefMut;
use std::rc::Rc;

/// Input
///  - Single batch of mines to be routed together
/// Output
///  - Each mine has 4 destinations
///  - Therefore batch has 4^n possible combinations
///  - Combinations each can be permutated generating n! combinations
pub fn get_possible_routes_for_batch(
    surface: &VSurface,
    MineSelectBatch {
        mines,
        mut base_sources,
    }: MineSelectBatch,
) -> CompletePlan {
    // let mines_len = mines.len();
    // let mines_destinations_len: usize = mines.iter().map(|v| v.destinations().len()).sum();
    // info!(
    //     "Expanded {} mines with {} destinations to...",
    //     mines_len, mines_destinations_len,
    // );
    assert!(!mines.is_empty(), "nope");

    let mine_combinations = find_all_combinations(mines);
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
        VPoint::new(
            fixed_radius - TILES_PER_CHUNK as i32,
            fixed_radius - TILES_PER_CHUNK as i32,
        ),
    );

    // Did we actually generate unique steps?
    // let mut dedupe_test = mine_combinations.iter().collect_vec();
    // dedupe_test.sort();
    // dedupe_test.dedup();
    // let dedupe_len = dedupe_test.len();
    // assert_eq!(total_combinations_permut, dedupe_len);

    let sequences = build_routes_from_destinations(
        mine_combinations,
        fixed_finding_limiter,
        &mut base_sources.borrow_mut(),
    );
    CompletePlan {
        sequences,
        base_sources,
    }
}

pub struct CompletePlan {
    pub sequences: Vec<ExecutionSequence>,
    pub base_sources: Rc<RefCell<BaseSourceEighth>>,
}

#[derive(Clone, PartialOrd, Ord, PartialEq, Eq)]
struct PartialEntry {
    location: MineLocation,
    destination: VPointDirectionQ,
}

/// Find all combinations of `a[1,2,3,4], b[1,2,3,4], ... = [a1, b1], [a2, b2], ...`
/// This is <4^n sized Vec, because of the 4 possible choices.
///
/// Start with a list of mines with 4x possible positions.
/// Create combinations of `[a1, b1, c2, ...]`
fn find_all_combinations(mines: Vec<MineLocation>) -> Vec<Vec<PartialEntry>> {
    let mut routes: Vec<Vec<PartialEntry>> = Vec::new();
    for i in 0..4 {
        let mut route = Vec::new();
        for mine in &mines {
            if let Some(destination) = mine.destinations().get(i) {
                route.push(PartialEntry {
                    destination: *destination,
                    location: mine.clone(),
                })
            }
        }
        if !route.is_empty() {
            routes.push(route);
        }
    }
    routes
}

/// Find all re-ordered permutations of `[a,b,c,...] = n!`
/// This is huge
fn find_all_permutations(input_combinations: Vec<Vec<PartialEntry>>) -> Vec<Vec<PartialEntry>> {
    let mut permutated = Vec::new();
    for combination in input_combinations {
        let destinations_len = combination.len();
        for permutation in combination.into_iter().permutations(destinations_len) {
            permutated.push(permutation);
        }
    }
    permutated
}

/// Add the base source rail going to the destination, in order
fn build_routes_from_destinations(
    input_combinations: Vec<Vec<PartialEntry>>,
    fixed_finding_limiter: VArea,
    base_source: &mut BaseSourceEighth,
) -> Vec<ExecutionSequence> {
    let mut sequences: Vec<ExecutionSequence> = Vec::new();
    for combination in input_combinations {
        let mut routes: Vec<ExecutionRoute> = Vec::new();

        for (
            i,
            PartialEntry {
                destination,
                location,
            },
        ) in combination.into_iter().enumerate()
        {
            routes.push(ExecutionRoute {
                segment: base_source
                    .peek_at(i)
                    .segment_for_mine(&destination, &location),
                location,
                finding_limiter: fixed_finding_limiter.clone(),
            })
        }
        sequences.push(ExecutionSequence { routes });
    }
    sequences
}
