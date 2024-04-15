use crate::navigator::mori::{Rail, RailDirection};
use crate::navigator::rail_point_compare::RailPointCompare;
use crate::surface::surface::Surface;
use crate::surfacev::varea::VArea;
use crate::surfacev::vpoint::VPoint;
use crate::surfacev::vsurface::VSurface;
use crossbeam::queue::SegQueue;
use opencv::core::add;
use tracing::trace;

/// Pathfinder v2, Josuiji Shinri
///
/// Specialized algorithm for "expanding L" or "axis hugging",
/// without ambiguous expensive Mori astar/BFS/DFS algorithm.
///
/// Generate network by
/// * Go straight to the edge of map. Crossed patches are ignored
/// * Turn and navigate to next closest perpendicular patch
/// * Continue until unplaceable rail
/// * Go back, turn and try to go around
/// * Select lowest possible solutions
///
/// ```text
/// ┌───────┐    ┌───────┐    ┌───────┐
/// │s xxxxx│    │s xxxxx│    │s xxxxx│
/// │      x│    │     xx│    │  xxx x│
/// │      x│    │     xx│    │    x x│
/// │      x│ -> │     xx│ or │    x x│
/// │      x│    │     xx│    │ xxxx x│
/// │      x│    │     xx│    │ x    x│
/// │      x│    │     xx│    │ xxxx x│
/// └───────┘    └───────┘    └───────┘
/// ```
///
pub fn shinri_start(surface: &Surface, start: Rail, end: Rail) -> Option<Vec<Rail>> {
    let mut path = vec![RailPointCompare::new(start)];

    None
}

fn navigate_axis_until<FS>(
    surface: &VSurface,
    path: &mut Vec<RailPointCompare>,
    test_navigate_success: FS,
    search_area: &VArea,
    end: &Rail,
) where
    FS: Fn(&RailPointCompare) -> bool,
{
    let straight_result =
        navigate_straight_until(surface, path, test_navigate_success, search_area, end);
    // match straight_result {
    //     StraightResult::NeedGoAround =>
    //
    // }
}

enum StraightResult {
    NeedGoAround,
    NavigateSuccess,
}

fn navigate_straight_until<FS>(
    surface: &VSurface,
    path: &mut Vec<RailPointCompare>,
    test_navigate_success: FS,
    search_area: &VArea,
    end: &Rail,
) -> StraightResult
where
    FS: Fn(&RailPointCompare) -> bool,
{
    loop {
        let edge = path.last().unwrap();

        let Some(next) =
            edge.inner
                .move_forward_step()
                .into_buildable(surface, search_area, path, end)
        else {
            trace!("Hit issue at {:?}", edge.inner);
            return StraightResult::NeedGoAround;
        };

        let next_compare = RailPointCompare::new(next);
        let should_stop = test_navigate_success(&next_compare);

        path.push(next_compare);

        if should_stop {
            return StraightResult::NavigateSuccess;
        }
    }
}

enum GoAroundMachine {
    FirstLeg(u32),
    Across(u32),
    LastLeg(u32),
}

fn navigate_around<FS>(
    surface: &VSurface,
    path: &mut Vec<RailPointCompare>,
    test_navigate_success: FS,
    search_area: &VArea,
    end: &Rail,
) where
    FS: Fn(&RailPointCompare) -> bool,
{
    let mut new_path = Vec::new();
    let mut pop_existing = false;

    'machine: loop {
        let mut state_stack = vec![GoAroundMachine::FirstLeg(0)];
        match state_stack.last().unwrap() {
            GoAroundMachine::FirstLeg(steps) => {
                if pop_existing {
                    path.pop().unwrap();
                    pop_existing = false;
                }
                let axis_rail = &path.last().unwrap().inner;

                let mut added_rail = Vec::new();

                let leg_turn_first_owned =
                    axis_rail
                        .move_left()
                        .into_buildable(surface, search_area, path, end);
                match leg_turn_first_owned {
                    Some(next) => {
                        added_rail.push(next);
                    }
                    None => {
                        // can't even turn, go back
                        pop_existing = true;
                        continue 'machine;
                    }
                };

                let steps = steps + 1;
                for step in 0..steps {
                    let leg_straight = added_rail
                        .last()
                        .unwrap()
                        .move_forward_step()
                        .into_buildable(surface, search_area, path, end);
                    match leg_straight {
                        Some(next) => {
                            added_rail.push(next);
                        }
                        None => {
                            // can't turn and go up, go back
                            pop_existing = true;
                            continue 'machine;
                        }
                    }
                }

                let leg_turn_last = added_rail.last().unwrap().move_right().into_buildable(
                    surface,
                    search_area,
                    path,
                    end,
                );
                match leg_turn_last {
                    Some(next) => {
                        added_rail.push(next);
                    }
                    None => {
                        // can't turn and go up and turn again, go forward one more
                        continue 'machine;
                    }
                }

                new_path.push(added_rail);
                replace_end_of_slice(&mut state_stack, GoAroundMachine::FirstLeg(steps));
            }
            GoAroundMachine::Across(steps) => {
                let previous_leg_turn = new_path.last().unwrap().last().unwrap();

                let mut added_rail = Vec::new();

                let steps = steps + 1;
                let mut next_rail = previous_leg_turn;
                for step in 0..steps {
                    let leg_straight = next_rail.move_forward_step().into_buildable(
                        surface,
                        search_area,
                        path,
                        end,
                    );
                    match leg_straight {
                        Some(leg_straight) => {
                            added_rail.push(leg_straight);
                            next_rail = &leg_straight;
                        }
                        None => {
                            // can't go across, need first leg to go up one more
                            continue 'machine;
                        }
                    }
                }
            }
            GoAroundMachine::LastLeg(steps) => {}
        }
    }
}

fn replace_end_of_slice<T>(slice: &mut [T], new_value: T) {
    assert!(!slice.is_empty(), "slice is empty");
    slice[slice.len() - 1] = new_value;
}
