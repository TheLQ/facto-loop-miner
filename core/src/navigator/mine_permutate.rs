use crate::navigator::mine_selector::MineSelectBatch;
use crate::navigator::path_side::BaseSourceEighth;
use crate::surfacev::mine::MineLocation;
use crate::surfacev::vsurface::VSurface;
use facto_loop_miner_fac_engine::common::varea::VArea;
use facto_loop_miner_fac_engine::common::vpoint::{VPoint, VPOINT_ONE};
use facto_loop_miner_fac_engine::common::vpoint_direction::VPointDirectionQ;
use facto_loop_miner_fac_engine::game_entities::direction::FacDirectionQuarter;
use facto_loop_miner_fac_engine::game_entities::rail_straight::RAIL_STRAIGHT_DIAMETER_I32;
use itertools::Itertools;
use simd_json::prelude::ArrayTrait;
use tracing::info;

/// Solve 2 core problems
/// - Get an ordered list of patches to navigate to
/// - (!) Provide multiple possible solutions when, due to patching problems,
///   - We cannot reach a patch anymore and must discard it
///   - Another corner creates a more optimal/lower-cost path
///   - Different order creates a more optimal/lower-cost path
///
/// Total paths = N * ____
pub fn get_possible_routes_for_batch(
    surface: &VSurface,
    MineSelectBatch {
        mines,
        base_sources,
    }: MineSelectBatch,
) -> Vec<PlannedBatch> {
    let mine_choices: Vec<MineChoices> = mines
        .into_iter()
        .map(|mine| MineChoices::from_mine(surface, mine))
        .collect();
    let mine_choice_len = mine_choices.len();
    let mine_choice_destinations_len: usize =
        mine_choices.iter().map(|v| v.destinations.len()).sum();
    info!(
        "Expanded {} mines with {} destinations to...",
        mine_choice_len, mine_choice_destinations_len,
    );

    let mine_combinations = find_all_combinations(mine_choices);
    let total_combinations_base = mine_combinations.len();
    info!("generated {} combinations", total_combinations_base);
    let mine_combinations = find_all_permutations(mine_combinations);
    let total_combinations_permut = mine_combinations.len();
    info!("generated {} permutations", total_combinations_permut);

    info!(
        "Expanded {} mines with {} destinations to {} combinations then {} permutated",
        mine_choice_len,
        mine_choice_destinations_len,
        total_combinations_base,
        total_combinations_permut
    );

    let routes = build_routes_from_destinations(mine_combinations, base_sources);
    routes
}

pub struct PlannedRoute {
    pub location: MineLocation,
    pub destination: VPointDirectionQ,
    pub base_source: VPointDirectionQ,
}

pub struct PlannedBatch {
    pub routes: Vec<PlannedRoute>,
}

struct MineChoices {
    location: MineLocation,
    destinations: Vec<VPointDirectionQ>,
}

#[derive(Clone)]
struct PartialEntry {
    location: MineLocation,
    destination: VPointDirectionQ,
}

/// Find all combinations of `a[1,2,3,4], b[1,2,3,4], ... = [a1, b1], [a2, b2], ...`
/// This is <4^n sized Vec, because of the 4 possible choices.
///
/// Start with a list of mines with 4x possible positions.
/// Create combinations of `[a1, b1, c2, ...]`
fn find_all_combinations(mines_choices: Vec<MineChoices>) -> Vec<Vec<PartialEntry>> {
    let mut routes: Vec<Vec<PartialEntry>> = Vec::new();
    for i in 0..4 {
        let mut route = Vec::new();
        for choice in &mines_choices {
            if let Some(destination) = choice.destinations.get(i) {
                route.push(PartialEntry {
                    destination: *destination,
                    location: choice.location.clone(),
                })
            }
        }
        routes.push(route);
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
    base_sources: Vec<VPointDirectionQ>,
) -> Vec<PlannedBatch> {
    let mut batches: Vec<PlannedBatch> = Vec::new();
    for combination in input_combinations {
        let mut routes: Vec<PlannedRoute> = Vec::new();
        let mut base_sources = base_sources.iter();

        for PartialEntry {
            destination,
            location,
        } in combination
        {
            routes.push(PlannedRoute {
                destination,
                location,
                base_source: *base_sources.next().unwrap(),
            })
        }
        batches.push(PlannedBatch { routes });
    }
    batches
}

impl MineChoices {
    // For an individual patch-group, come up with multiple choices for
    fn from_mine(surface: &VSurface, location: MineLocation) -> Self {
        let mut destinations: Vec<VPointDirectionQ> = Vec::new();

        let mine_area = expanded_mine_no_touching_zone(surface, &location);
        // centered top
        {
            let mut centered_point = VPoint::new(mine_area.point_center().x(), mine_area.start.y());
            // Go back up width of rail + inner-rail space
            centered_point = centered_point.move_round2_down() + VPOINT_ONE;
            tracing::trace!("testing top left     {centered_point} from {mine_area:?}");

            if !surface.is_point_out_of_bounds(&centered_point) {
                destinations.push(VPointDirectionQ(centered_point, FacDirectionQuarter::East));
                destinations.push(VPointDirectionQ(centered_point, FacDirectionQuarter::West));
            }
        };
        {
            let mut centered_point = VPoint::new(
                mine_area.point_center().x(),
                mine_area.point_bottom_right().y(),
            );
            // Go back up width of rail + inner-rail space
            centered_point = centered_point.move_round2_down() + VPOINT_ONE;
            tracing::trace!("testing bottom right {centered_point} from {mine_area:?}");

            if !surface.is_point_out_of_bounds(&centered_point) {
                destinations.push(VPointDirectionQ(centered_point, FacDirectionQuarter::East));
                destinations.push(VPointDirectionQ(centered_point, FacDirectionQuarter::West));
            }
        };

        // remove out of bounds
        // destinations.retain(|rail| !surface.is_point_out_of_bounds(&rail.0));
        assert!(
            !destinations.is_empty(),
            "stripped all possible destinations from {location:?} in {surface}"
        );

        // TODO old removes: optimizations to remove impossible situations

        Self {
            location,
            destinations,
        }
    }
}

const MINE_RAIL_BUFFER_PIXELS: i32 = RAIL_STRAIGHT_DIAMETER_I32 * 2 * 2;

fn expanded_mine_no_touching_zone(surface: &VSurface, mine: &MineLocation) -> VArea {
    let area = &mine.area;

    VArea::from_arbitrary_points_pair(
        area.start
            .move_xy(-MINE_RAIL_BUFFER_PIXELS, -MINE_RAIL_BUFFER_PIXELS)
            .trim_max(surface.point_top_left()),
        area.point_bottom_right()
            .move_xy(MINE_RAIL_BUFFER_PIXELS, MINE_RAIL_BUFFER_PIXELS)
            .trim_min(surface.point_bottom_right()),
    )
}
