use crate::navigator::mori::{Rail, RailDirection, RailMode, RAIL_STEP_SIZE_I32};
use crate::navigator::path_grouper::{MineBase, MineBaseBatch};
use crate::state::machine_v1::CENTRAL_BASE_TILES;
use crate::surfacev::varea::VArea;
use crate::surfacev::vpoint::{VPoint, SHIFT_POINT_EIGHT, SHIFT_POINT_ONE};
use crate::surfacev::vsurface::VSurface;
use itertools::Itertools;
use tracing::debug;

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
    base_source: &BaseSourceSide,
) -> MineRouteCombinationBatch {
    let mine_choices: Vec<MineChoices> = mine_batch
        .mines
        .into_iter()
        .map(|mine| MineChoices::from_mine(surface, mine))
        .collect();

    let mine_combinations = find_all_combinations(mine_choices);
    let mine_combinations = find_all_permutations(mine_combinations);

    let mut route_combinations = Vec::new();
    destinations_to_route(
        mine_combinations,
        base_source,
        mine_batch.base_direction,
        &mut route_combinations,
    );

    MineRouteCombinationBatch {
        combinations: route_combinations,
    }
}

struct MineChoices {
    mine: MineBase,
    destinations: Vec<Rail>,
}

#[derive(Clone)]
struct MineDestination {
    mine: MineBase,
    entry_rail: Rail,
}

#[derive(Clone)]
struct MineDestinationCombination {
    destinations: Vec<MineDestination>,
}

pub struct MineRouteEndpoints {
    pub mine: MineBase,
    pub entry_rail: Rail,
    pub base_rail: Rail,
}

pub struct MineRouteCombination {
    pub routes: Vec<MineRouteEndpoints>,
}

pub struct MineRouteCombinationBatch {
    pub combinations: Vec<MineRouteCombination>,
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
        } else {
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
    let mut permutated_combinations = Vec::new();
    for combination in input_combinations {
        for permutation in combination
            .destinations
            .iter()
            .permutations(combination.destinations.len())
        {
            permutated_combinations.push(MineDestinationCombination {
                destinations: permutation.into_iter().cloned().collect(),
            });
        }
    }
    permutated_combinations
}

/// Add the base source rail going to the destination, in order of the Vec
fn destinations_to_route(
    mine_combinations: Vec<MineDestinationCombination>,
    base_source: &BaseSourceSide,
    base_direction: RailDirection,
    route_combinations: &mut Vec<MineRouteCombination>,
) {
    for mine_combination in mine_combinations {
        let routes = mine_combination
            .destinations
            .into_iter()
            .enumerate()
            .map(|(index, destination)| {
                destination.into_mine_route(Rail {
                    endpoint: base_source.peek_add(index),
                    direction: base_direction.clone(),
                    mode: RailMode::Straight,
                })
            })
            .collect();
        route_combinations.push(MineRouteCombination { routes })
    }
}

fn get_expanded_patch_points(area: &VArea) -> (VPoint, VPoint) {
    // main corners
    let mut patch_top_left = area.start.move_round16_down() + SHIFT_POINT_ONE;
    patch_top_left.assert_odd_16x16_position();

    let mut patch_bottom_right = area.point_bottom_left().move_round16_up() + SHIFT_POINT_ONE;
    patch_bottom_right.assert_odd_16x16_position();

    for _ in 0..2 {
        patch_top_left = patch_top_left - SHIFT_POINT_EIGHT;
        patch_bottom_right = patch_bottom_right + SHIFT_POINT_EIGHT;
    }

    (patch_top_left, patch_bottom_right)
}

impl MineChoices {
    fn from_mine(surface: &VSurface, mine: MineBase) -> Self {
        let mut destinations = Vec::new();
        let (patch_top_left, patch_bottom_right) = get_expanded_patch_points(&mine.area);

        destinations.push(Rail::new_straight(patch_top_left, RailDirection::Right));
        destinations.push(Rail::new_straight(patch_bottom_right, RailDirection::Left));

        // opposite corners
        let patch_bottom_left = VPoint::new(patch_top_left.x(), patch_bottom_right.y());
        let patch_top_right = VPoint::new(patch_bottom_right.x(), patch_top_left.y());

        destinations.push(Rail::new_straight(patch_bottom_left, RailDirection::Right));
        destinations.push(Rail::new_straight(patch_top_right, RailDirection::Left));

        destinations.retain(|rail| rail.is_area_buildable(surface));
        if destinations.len() != 4 {
            debug!("Reduced mine destinations from 4 to {}", destinations.len());
        }

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

impl MineDestination {
    fn into_mine_route(self, base_rail: Rail) -> MineRouteEndpoints {
        MineRouteEndpoints {
            mine: self.mine,
            entry_rail: self.entry_rail,
            base_rail,
        }
    }
}

const CENTRAL_BASE_TILES_BY_RAIL_STEP: i32 = CENTRAL_BASE_TILES
    + ((RAIL_STEP_SIZE_I32 * 2) - (CENTRAL_BASE_TILES % (RAIL_STEP_SIZE_I32 * 2)));

pub struct BaseSource {
    positive: BaseSourceSide,
    negative: BaseSourceSide,
}

/// Because a struct field of IntoIterator<VPoint> creates Rust type hell
struct BaseSourceSide {
    sign: i32,
    next: i32,
}

impl BaseSourceSide {
    pub fn next(&mut self) -> VPoint {
        let result = self.get_for_pos(self.next);
        self.next = self.next + 1;
        result
    }

    pub fn peek_add(&self, pos_add: usize) -> VPoint {
        self.get_for_pos(self.next + pos_add as i32)
    }

    // pub fn peek_add_vec(&self, pos_add: usize) -> Vec<VPoint> {
    //     let result = Vec::with_capacity(pos_add);
    //
    //     result
    // }

    pub fn get_for_pos(&self, pos: i32) -> VPoint {
        VPoint::new(
            CENTRAL_BASE_TILES_BY_RAIL_STEP,
            self.sign * pos * RAIL_STEP_SIZE_I32 * 2,
        ) + SHIFT_POINT_ONE
    }
}
