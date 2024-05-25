use crate::navigator::mori::{Rail, RailDirection};
use crate::navigator::path_grouper::{MineBase, MineBaseBatch};
use crate::surfacev::varea::VArea;
use crate::surfacev::vpatch::VPatch;
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
/// Total paths = N *
pub fn get_possible_routes_for_batch(
    surface: &VSurface,
    mine_batch: MineBaseBatch,
) -> MineRouteBatch {
    let mines: Vec<MineChoices> = mine_batch
        .mines
        .into_iter()
        .map(|mine| MineChoices::from_mine(surface, mine))
        .collect();

    let routes = find_all_combinations(mines);
    let routes = find_all_permutations(routes);

    MineRouteBatch { routes }
}

struct MineChoices {
    mine: MineBase,
    destinations: Vec<Rail>,
}

#[derive(Clone)]
pub struct MineDestination {
    mine: MineBase,
    entry_rail: Rail,
}

#[derive(Clone)]
pub struct MineRoute {
    pub mines: Vec<MineDestination>,
}

pub struct MineRouteBatch {
    pub routes: Vec<MineRoute>,
}

/// Find all combinations of `a[1,2,3,4], b[1,2,3,4], ... = 4^n` sized Vec.
/// This is huge.
fn find_all_combinations(mines_choices: Vec<MineChoices>) -> Vec<MineRoute> {
    let mut routes = Vec::new();
    for mine_choices in mines_choices {
        if routes.is_empty() {
            // seed
            for mine_destination in mine_choices.to_mine_destinations() {
                routes.push(MineRoute {
                    mines: vec![mine_destination],
                });
            }
        } else {
            // every existing route is cloned 4x for the new destinations
            let mut new_routes = Vec::new();
            for route_existing in routes {
                for mine_destination in mine_choices.to_mine_destinations() {
                    let mut route_new = route_existing.clone();
                    route_new.mines.push(mine_destination);
                    new_routes.push(route_new);
                }
            }
            routes = new_routes;
        }
    }
    routes
}

/// Find all re-ordered permutations of `[a,b,c,...] = n!`
fn find_all_permutations(input_routes: Vec<MineRoute>) -> Vec<MineRoute> {
    let input_len = input_routes.len();
    let mut result = Vec::new();
    for input_route in input_routes {
        for permutation in input_route.mines.iter().permutations(input_len) {
            result.push(MineRoute {
                mines: permutation.into_iter().cloned().collect(),
            });
        }
    }
    result
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
    // fn from_patch_block(block: &PatchOutpostAll, index: usize) -> Self {
    //     PatchOutpost {
    //         patch_indexes: block.patch_indexes.clone(),
    //         area: block.area.clone(),
    //         entry_rail: block.destinations[index].clone(),
    //     }
    // }
}
