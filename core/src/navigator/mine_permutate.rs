use crate::navigator::mine_selector::MineSelectBatch;
use crate::surfacev::mine::MineLocation;
use crate::surfacev::vsurface::VSurface;
use facto_loop_miner_fac_engine::common::varea::VArea;
use facto_loop_miner_fac_engine::common::vpoint::VPoint;
use facto_loop_miner_fac_engine::common::vpoint_direction::VPointDirectionQ;
use facto_loop_miner_fac_engine::constants::TILES_PER_CHUNK;
use facto_loop_miner_fac_engine::game_entities::direction::FacDirectionQuarter;
use facto_loop_miner_fac_engine::game_entities::rail_straight::RAIL_STRAIGHT_DIAMETER_I32;
use itertools::Itertools;
use simd_json::prelude::ArrayTrait;
use tracing::info;

/// Input
///  - Single batch of mines to be routed together
/// Output
///  - Each mine has 4 destinations
///  - Batch has 4^n possible combinations
///  - Combinations can be permutated generated n! combinations
///  - Every combination is sourced from the same list of base sources
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
    // info!(
    //     "Expanded {} mines with {} destinations to...",
    //     mine_choice_len, mine_choice_destinations_len,
    // );
    assert!(!mine_choices.is_empty(), "nope");

    let mine_combinations = find_all_combinations(mine_choices);
    assert!(!mine_combinations.is_empty(), "nope");
    let total_combinations_base = mine_combinations.len();
    // info!("generated {} combinations", total_combinations_base);
    let mine_combinations = find_all_permutations(mine_combinations);
    let total_combinations_permut = mine_combinations.len();
    // info!("generated {} permutations", total_combinations_permut);

    info!(
        "Expanded {} mines with {} destinations to {} combinations then {} permutated",
        mine_choice_len,
        mine_choice_destinations_len,
        total_combinations_base,
        total_combinations_permut
    );

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

    let routes =
        build_routes_from_destinations(mine_combinations, base_sources, fixed_finding_limiter);
    routes
}

pub struct PlannedRoute {
    pub location: MineLocation,
    pub destination: VPointDirectionQ,
    pub base_source: VPointDirectionQ,
    pub finding_limiter: VArea,
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
    base_sources: Vec<VPointDirectionQ>,
    fixed_finding_limiter: VArea,
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
                finding_limiter: fixed_finding_limiter.clone(),
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
            centered_point = centered_point.move_round_rail_down();

            // let y_diff = centered_point.y() - location.area.start.y();
            // if y_diff > 20 {
            //     tracing::warn!("diff {y_diff}")
            // }

            if !surface.is_point_out_of_bounds(&centered_point) {
                destinations.push(VPointDirectionQ(centered_point, FacDirectionQuarter::East));
                // destinations.push(VPointDirectionQ(centered_point, FacDirectionQuarter::West));
            }
        };
        {
            let mut centered_point = VPoint::new(
                mine_area.point_center().x(),
                mine_area.point_bottom_right().y(),
            );
            centered_point = centered_point.move_round_rail_up();

            if !surface.is_point_out_of_bounds(&centered_point) {
                destinations.push(VPointDirectionQ(centered_point, FacDirectionQuarter::East));
                // destinations.push(VPointDirectionQ(centered_point, FacDirectionQuarter::West));
            }
        };

        // remove out of bounds
        destinations.retain(|rail| !surface.is_point_out_of_bounds(&rail.0));
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
