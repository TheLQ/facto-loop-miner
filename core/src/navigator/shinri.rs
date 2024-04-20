use crate::navigator::mori::{Rail, RailDirection, TurnType};
use crate::navigator::rail_point_compare::RailPointCompare;
use crate::surface::surface::Surface;
use crate::surfacev::varea::VArea;
use crate::surfacev::vpoint::VPoint;
use crate::surfacev::vsurface::VSurface;
use crossbeam::queue::SegQueue;
use itertools::Itertools;
use opencv::core::add;
use strum::Display;
use tracing::{info, trace};

/// Pathfinder v2, Josuiji Shinri
///
/// Specialized algorithm for "expanding L" or "axis hugging",
/// without ambiguous expensive v1 Mori astar/BFS/DFS algorithm.
///
/// Generate network by
/// * Go straight to the edge of map. Crossed patches are ignored
/// * Turn and navigate to next closest perpendicular patch
/// * Continue until unplaceable rail
/// * Go back, turn and try to go around, coming back to axis
/// * Repeat until reaching goal
/// * Select lowest possible solutions
///
/// ```text
/// From S > P
/// ┌───────┐    ┌───────┐    ┌───────┐
/// │s xxxxx│    │s xxxxx│    │s xxxxx│
/// │      x│    │     xx│    │  xxx x│
/// │      x│    │     xx│    │    x x│
/// │      x│ -> │     xx│ or │ xxxx x│
/// │      x│    │     xx│    │ x    x│
/// │      x│    │     xx│    │ xxxx P│
/// │      P│    │     PP│    │    P P│
/// └───────┘    └───────┘    └───────┘
/// ```
///
pub fn shinri_start(
    surface: &VSurface,
    start: Rail,
    end: Rail,
    search_area: &VArea,
) -> Option<Vec<Rail>> {
    let mut path = vec![RailPointCompare::new(start)];

    let test_x_success = |p: &RailPointCompare| p.inner.endpoint.x() >= end.endpoint.x();
    let test_x_success = |p: &RailPointCompare| p.inner.endpoint.x() >= end.endpoint.x();

    navigate_axis_until(surface, &mut path, &test_x_success, search_area, &end);
    navigate_around(surface, &mut path, &test_x_success, search_area, &end);

    // if test_x_success(&path.last().unwrap()) {}

    Some(path.into_iter().map(|c| c.inner).collect())
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

struct ShinriLeg {
    steps: u32,
    turn: TurnType,
}

impl ShinriLeg {
    fn try_advance() {}
}

enum TryAdvanceResult {}

struct ShinriKansen {
    legs: Vec<ShinriLeg>,
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

        trace!("advancing {:?}", next_compare.inner);
        path.push(next_compare);

        if should_stop {
            return StraightResult::NavigateSuccess;
        }
    }
}

#[derive(PartialEq, Display)]
enum GoAroundMachine {
    FirstLeg,
    Across,
    LastLeg,
}

/// Go perpendicular to axis, across, then back
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

    let mut first_leg_steps = 0;
    let mut across_steps = 0;
    let mut last_leg_steps = 0;

    let mut state_stack = vec![GoAroundMachine::FirstLeg];
    'machine: loop {
        trace!(
            "State Stack {}",
            state_stack.iter().map(|v| format!("{}", v)).join(",")
        );
        match state_stack.last().unwrap() {
            GoAroundMachine::FirstLeg => {
                trace!(
                    "[FirstLeg] making straight {} pop_existing {}",
                    first_leg_steps + 1,
                    pop_existing
                );
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
                        trace!("[FirstLeg] [FirstTurn] failed, move GoAround back");
                        pop_existing = true;
                        continue 'machine;
                    }
                };

                first_leg_steps += 1;
                for step in 0..first_leg_steps {
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
                            trace!("[FirstLeg] [Straight{}] failed, move GoAround back", step);
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
                        trace!("[FirstLeg] [SecondTurn] failed, more straight");
                        continue 'machine;
                    }
                }

                new_path.push(added_rail);
                state_stack.push(GoAroundMachine::Across);
                // trace!(
                //     "[FirstLeg] End for {}",
                //     state_stack.iter().map(|v| format!("{}", v)).join(",")
                // );
            }
            GoAroundMachine::Across => {
                trace!("[SecondLeg] making straight {}", across_steps + 1);
                let previous_leg_turn = new_path.last().unwrap().last().unwrap();

                let mut added_rail: Vec<Rail> = Vec::new();

                across_steps += 1;
                for step in 0..across_steps {
                    let leg_straight = added_rail
                        .last()
                        .or_else(|| new_path.last().unwrap().last())
                        .unwrap()
                        .move_forward_step()
                        .into_buildable(surface, search_area, path, end);
                    match leg_straight {
                        Some(leg_straight) => {
                            added_rail.push(leg_straight);
                        }
                        None => {
                            // can't go across, need first leg to go up one more
                            let pop_accross = state_stack.ends_with(&[GoAroundMachine::Across]);
                            if pop_accross {
                                state_stack.pop();
                            }
                            trace!("[SecondLeg] [Straight{}] failed, move GoAround back", step);
                            continue 'machine;
                        }
                    }
                }

                if !state_stack.ends_with(&[GoAroundMachine::Across]) {
                    state_stack.push(GoAroundMachine::Across);
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
                        // can't go across and turn, need to go across one more
                        trace!("[SecondLeg] [Turn] failed, more across");
                        continue 'machine;
                    }
                }

                new_path.push(added_rail);
                state_stack.push(GoAroundMachine::LastLeg);
            }
            GoAroundMachine::LastLeg => {
                trace!("[LastLeg] making straight {}", last_leg_steps + 1);
                let previous_leg_turn = new_path.last().unwrap().last().unwrap();

                let mut added_rail: Vec<Rail> = Vec::new();

                last_leg_steps += 1;
                for step in 0..last_leg_steps {
                    let leg_straight = added_rail
                        .last()
                        .or_else(|| new_path.last().unwrap().last())
                        .unwrap()
                        .move_forward_step()
                        .into_buildable(surface, search_area, path, end);
                    match leg_straight {
                        Some(leg_straight) => {
                            added_rail.push(leg_straight);
                        }
                        None => {
                            // can't go across, need first leg to go up one more
                            let pop_accross = state_stack.ends_with(&[GoAroundMachine::LastLeg]);
                            if pop_accross {
                                state_stack.pop();
                            }
                            trace!("[ThirdLeg] [Straight{}] failed, move GoAround back", step);
                            continue 'machine;
                        }
                    }
                }

                if !state_stack.ends_with(&[GoAroundMachine::LastLeg]) {
                    state_stack.push(GoAroundMachine::LastLeg);
                }

                let leg_turn_last = added_rail.last().unwrap().move_left().into_buildable(
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
                        // can't go across and turn, need to go across one more
                        trace!("[ThirdLeg] [Turn] failed, more third leg");
                        continue 'machine;
                    }
                }

                new_path.push(added_rail);
                info!(
                    "end of stack first {} second {} third {}",
                    first_leg_steps, across_steps, last_leg_steps
                );
                break;
            }
        }
    }

    for leg_paths in new_path {
        for leg_path in leg_paths {
            path.push(RailPointCompare::new(leg_path));
        }
    }
}

fn replace_end_of_slice<T>(slice: &mut [T], new_value: T) {
    assert!(!slice.is_empty(), "slice is empty");
    slice[slice.len() - 1] = new_value;
}
