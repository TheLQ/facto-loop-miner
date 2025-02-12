use crate::navigator::mine_selector::MineBaseBatch;
use crate::navigator::path_side::BaseSourceEighth;
use crate::surfacev::vsurface::VSurface;
use facto_loop_miner_fac_engine::common::varea::VArea;
use facto_loop_miner_fac_engine::common::vpoint::VPoint;
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
    mine_batch: MineBaseBatch,
) -> MineRouteCombinationBatch {
    let mine_choices: Vec<MineChoices> = mine_batch
        .mines
        .into_iter()
        .map(|mine| MineChoices::from_mine(surface, mine))
        .collect();
    let mine_choice_len = mine_choices.len();
    let mine_choice_destinations_len = mine_choices
        .iter()
        .fold(0, |total, v| total + v.destinations.len());
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

    let mut route_combinations = Vec::new();
    build_routes_from_destinations(
        mine_combinations,
        mine_batch.base_direction,
        &mine_batch.base_source_eighth.lock().unwrap(),
        &mut route_combinations,
    );
    // let before = route_combinations.len();
    // route_combinations = route_combinations.into_iter().unique().collect();
    // let after = route_combinations.len();
    // panic!("reduced from {} to {}", before, after);

    MineRouteCombinationBatch {
        combinations: route_combinations,
        planned_search_area: mine_batch.batch_search_area,
    }
}

pub struct MineChoices {
    mine: MineBase,
    pub destinations: Vec<Rail>,
}

#[derive(PartialEq, Eq, Hash, Clone)]
struct MineDestination {
    mine: MineBase,
    entry_rail: Rail,
}

#[derive(PartialEq, Eq, Hash, Clone)]
struct MineDestinationCombination {
    destinations: Vec<MineDestination>,
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct MineRouteEndpoints {
    pub mine: MineBase,
    pub entry_rail: Rail,
    pub base_rail: Rail,
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct MineRouteCombination {
    pub routes: Vec<MineRouteEndpoints>,
}

pub struct MineRouteCombinationBatch {
    pub combinations: Vec<MineRouteCombination>,
    pub planned_search_area: VArea,
}

/// Find all combinations of `a[1,2,3,4], b[1,2,3,4], ... = 4^n` sized Vec.
/// This is huge.
///
/// Start with a list of mines with 4x possible positions.
/// Create combinations of `[a1, b1, c2, ...]`
fn find_all_combinations(mines_choices: Vec<MineChoices>) -> Vec<MineDestinationCombination> {
    let mut routes = Vec::new();
    for mine_choices in mines_choices {
        if routes.is_empty() {
            // seed
            for mine_destination in mine_choices.to_mine_destinations() {
                routes.push(MineDestinationCombination {
                    destinations: vec![mine_destination],
                });
            }
        } else if !mine_choices.destinations.is_empty() {
            // every existing route is cloned 4x for the new destinations
            let mut new_routes = Vec::new();
            for route_existing in routes {
                for new_mine_destination in mine_choices.to_mine_destinations() {
                    let mut route_new = route_existing.clone();
                    route_new.destinations.push(new_mine_destination);
                    new_routes.push(route_new);
                }
            }
            routes = new_routes;
        }
    }
    routes
}

/// Find all re-ordered permutations of `[a,b,c,...] = n!`
fn find_all_permutations(
    input_combinations: Vec<MineDestinationCombination>,
) -> Vec<MineDestinationCombination> {
    let mut seen_lengths = Vec::new();

    let mut permutated_combinations = Vec::new();
    for combination in input_combinations {
        let destinations_len = combination.destinations.len();
        if !seen_lengths.contains(&destinations_len) {
            info!("Found lengths {}", destinations_len);
            seen_lengths.push(destinations_len);
        }

        for permutation in combination
            .destinations
            .into_iter()
            .permutations(destinations_len)
        {
            permutated_combinations.push(MineDestinationCombination {
                destinations: permutation.to_vec(),
            });
        }
    }
    permutated_combinations
}

/// Add the base source rail going to the destination, in order of the Vec
fn build_routes_from_destinations(
    mine_combinations: Vec<MineDestinationCombination>,
    base_direction: RailDirection,
    base_source_eighth: &BaseSourceEighth,
    route_combinations: &mut Vec<MineRouteCombination>,
) {
    for mine_combination in mine_combinations {
        let routes = mine_combination
            .destinations
            .into_iter()
            .enumerate()
            .map(|(index, destination)| {
                destination.into_mine_route(Rail {
                    endpoint: base_source_eighth.peek_add(index),
                    direction: base_direction.clone(),
                    mode: RailMode::Straight,
                })
            })
            .collect();
        route_combinations.push(MineRouteCombination { routes })
    }
}

pub const MINE_CHOICE_TRUNCATE_DESTINATIONS: usize = 2;

impl MineChoices {
    fn empty(mine: MineBase) -> Self {
        MineChoices {
            mine,
            destinations: Vec::new(),
        }
    }

    pub fn from_mine(surface: &VSurface, mine: MineBase) -> Self {
        let mut destinations: Vec<Rail> = Vec::new();

        let mine_area = expanded_mine_no_touching_zone(&mine);
        let top_center = {
            let mut centered_point =
                VPoint::new(mine_area.point_center().x(), mine_area.start.y()).move_round16_down();
            // Go back up width of rail + inner-rail space

            centered_point += SHIFT_POINT_ONE;
            centered_point.assert_odd_16x16_position();
            if surface.is_point_out_of_bounds(&centered_point) {
                return MineChoices::empty(mine);
            }
            let test_rail = Rail::new_straight(centered_point, RailDirection::Left);
            if !test_rail.is_area_buildable_fast(surface) {
                centered_point = centered_point.move_y(-16);
            }

            centered_point
        };
        let bottom_center = {
            let mut centered_point = VPoint::new(
                top_center.x() - /*Remove 1 odd shift*/1,
                mine_area.point_bottom_left().y(),
            )
            .move_round16_up();

            // Go back up width of rail + inner-rail space

            centered_point += SHIFT_POINT_ONE;
            centered_point.assert_odd_16x16_position();
            if surface.is_point_out_of_bounds(&centered_point) {
                return MineChoices::empty(mine);
            }
            let test_rail = Rail::new_straight(centered_point, RailDirection::Left);
            if !test_rail.is_area_buildable_fast(surface) {
                centered_point = centered_point.move_y(16);
            }
            centered_point
        };

        // top
        destinations.push(rail_move_forward_2x_then_180(
            top_center,
            RailDirection::Right,
        ));
        destinations.push(rail_move_forward_2x_then_180(
            top_center,
            RailDirection::Left,
        ));

        // bottom
        destinations.push(rail_move_forward_2x_then_180(
            bottom_center,
            RailDirection::Right,
        ));
        destinations.push(rail_move_forward_2x_then_180(
            bottom_center,
            RailDirection::Left,
        ));

        // pure out of bounds
        destinations.retain(|rail| !rail.area(surface).1);

        // destinations.retain(|rail| rail.is_area_buildable_fast(surface));

        let dummy_search_area = surface.dummy_area_entire_surface();
        destinations.retain(|rail| extend_rail_end(surface, &dummy_search_area, rail).is_some());

        if destinations.len() != 4 {
            info!("Reduced mine destinations from 4 to {}", destinations.len());
        }

        // TODO: OPTIMIZING ATTEMPT - pick closest 2 values.
        // TODO: Left rail only. Want closest side to source.
        // destinations.sort_by_key(|rail| (rail.endpoint.y(), rail.endpoint.x()));
        destinations.sort_by_key(|rail| rail.endpoint.x());
        destinations.truncate(MINE_CHOICE_TRUNCATE_DESTINATIONS);

        Self { mine, destinations }
    }

    // fn to_patch_outpost(&self, destination_index: usize) -> PatchOutpost {
    //     PatchOutpost {
    //         patch_indexes: self.patch_indexes.clone(),
    //         area: self.area.clone(),
    //         entry_rail: self.destinations[destination_index].clone()
    //     }
    // }

    fn to_mine_destinations(&self) -> Vec<MineDestination> {
        self.destinations
            .iter()
            .map(|destination| MineDestination {
                mine: self.mine.clone(),
                entry_rail: destination.clone(),
            })
            .collect()
    }
}

fn rail_move_forward_2x_then_180(start_point: VPoint, start_direction: RailDirection) -> Rail {
    Rail::new_straight(start_point, start_direction)
        .move_forward_step()
        .move_forward_step()
        .move_force_rotate_clockwise(2)
}

pub const MINE_RAIL_BUFFER_PIXELS: i32 = RAIL_STEP_SIZE_I32 * 2 * 2;

pub fn expanded_mine_no_touching_zone(mine: &MineBase) -> VArea {
    let area = &mine.area;
    VArea::from_arbitrary_points_pair(
        area.start
            .move_xy(-MINE_RAIL_BUFFER_PIXELS, -MINE_RAIL_BUFFER_PIXELS),
        area.point_bottom_left()
            .move_xy(MINE_RAIL_BUFFER_PIXELS, MINE_RAIL_BUFFER_PIXELS),
    )
}

impl MineDestination {
    fn into_mine_route(self, base_rail: Rail) -> MineRouteEndpoints {
        MineRouteEndpoints {
            mine: self.mine,
            entry_rail: self.entry_rail,
            base_rail,
        }
    }
}
