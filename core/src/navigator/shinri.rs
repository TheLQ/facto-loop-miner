use crate::navigator::mori::{Rail, RailDirection, TurnType};
use crate::navigator::rail_point_compare::RailPointCompare;
use crate::surface::surface::Surface;
use crate::surfacev::varea::VArea;
use crate::surfacev::vpoint::VPoint;
use crate::surfacev::vsurface::VSurface;
use crossbeam::queue::SegQueue;
use itertools::Itertools;
use opencv::core::add;
use std::path::Path;
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

    // if test_x_success(&path.last().unwrap()) {}

    Some(path.into_iter().map(|c| c.inner).collect())
}

fn navigate_axis_until<FS>(
    surface: &VSurface,
    path: &mut Vec<RailPointCompare>,
    test_navigate_success: &FS,
    search_area: &VArea,
    end: &Rail,
) where
    FS: Fn(&RailPointCompare) -> bool,
{
    loop {
        trace!("starting axis");

        navigate_straight_until(surface, path, test_navigate_success, search_area, end);
        if test_navigate_success(path.last().unwrap()) {
            trace!("navigated axis!");
            break;
        }

        let pre_around_length = path.len();
        navigate_around(surface, path, test_navigate_success, search_area, &end);

        if test_navigate_success(path.last().unwrap()) {
            panic!("no don't want this!");
            break;
        }
    }

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
    test_navigate_success: &FS,
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

#[derive(PartialEq, Display, Clone)]
enum GoAroundMachine {
    FirstLeg,
    Across,
    LastLeg,
}

impl GoAroundMachine {
    fn get_index(&self, state: &[GoAroundState; 3]) -> usize {
        match self {
            GoAroundMachine::FirstLeg => 0,
            GoAroundMachine::Across => 1,
            GoAroundMachine::LastLeg => 2,
        }
    }

    // fn get_state_mut<'a>(&self, state: &'a mut [GoAroundState; 3]) -> &'a mut GoAroundState {
    //     match self {
    //         GoAroundMachine::FirstLeg => state.get_mut(0).unwrap(),
    //         GoAroundMachine::Across => state.get_mut(1).unwrap(),
    //         GoAroundMachine::LastLeg => state.get_mut(2).unwrap(),
    //     }
    // }

    fn get_steps(&self, state: &[GoAroundState; 3]) -> u32 {
        state[self.get_index(state)].steps
    }

    fn incriment_steps(&self, state: &mut [GoAroundState; 3]) {
        state[self.get_index(state)].steps += 1
    }

    fn get_path<'a>(&self, state: &'a mut [GoAroundState; 3]) -> &'a mut Vec<Rail> {
        &mut state[self.get_index(state)].path
    }

    fn get_previous_path_end<'a>(&self, state: &'a [GoAroundState; 3]) -> &'a Rail {
        state[self.get_index(state) - 1].path.last().unwrap()
    }
}

struct GoAroundState {
    path: Vec<Rail>,
    steps: u32,
}

impl GoAroundState {
    pub fn new() -> Self {
        Self {
            path: Vec::new(),
            steps: 0,
        }
    }
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
    let mut pop_existing = false;

    let mut machine_state: [GoAroundState; 3] = [
        GoAroundState::new(),
        GoAroundState::new(),
        GoAroundState::new(),
    ];

    let mut machine_modes = vec![GoAroundMachine::FirstLeg];
    'machine: loop {
        trace!(
            "State Stack {}",
            machine_modes.iter().map(|v| format!("{}", v)).join(",")
        );
        let mut added_rail = Vec::new();

        // todo: clone is horrible
        let machine_mode = machine_modes.last().unwrap().clone();
        match &machine_mode {
            GoAroundMachine::FirstLeg => {
                trace!(
                    "[FirstLeg] making straight {} pop_existing {}",
                    machine_mode.get_steps(&machine_state) + 1,
                    pop_existing
                );
                if pop_existing {
                    path.pop().unwrap();
                    pop_existing = false;
                }
                let axis_rail = &path.last().unwrap().inner;

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

                machine_mode.incriment_steps(&mut machine_state);
                for step in 0..machine_mode.get_steps(&machine_state) {
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

                machine_mode.get_path(&mut machine_state).clear();
                machine_mode
                    .get_path(&mut machine_state)
                    .append(&mut added_rail);
                machine_modes.push(GoAroundMachine::Across);
                // trace!(
                //     "[FirstLeg] End for {}",
                //     state_stack.iter().map(|v| format!("{}", v)).join(",")
                // );
            }
            GoAroundMachine::Across => {
                trace!(
                    "[SecondLeg] making straight {}",
                    machine_mode.get_steps(&machine_state) + 1
                );

                machine_mode.incriment_steps(&mut machine_state);
                for step in 0..machine_mode.get_steps(&machine_state) {
                    let leg_straight = added_rail
                        .last()
                        .unwrap_or_else(|| machine_mode.get_previous_path_end(&machine_state))
                        .move_forward_step()
                        .into_buildable(surface, search_area, path, end);
                    match leg_straight {
                        Some(leg_straight) => {
                            added_rail.push(leg_straight);
                        }
                        None => {
                            // can't go across, need first leg to go up one more
                            let pop_accross = machine_modes.ends_with(&[GoAroundMachine::Across]);
                            if pop_accross {
                                machine_modes.pop();
                            }
                            trace!("[SecondLeg] [Straight{}] failed, move GoAround back", step);
                            continue 'machine;
                        }
                    }
                }

                if !machine_modes.ends_with(&[GoAroundMachine::Across]) {
                    machine_modes.push(GoAroundMachine::Across);
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

                machine_mode.get_path(&mut machine_state).clear();
                machine_mode
                    .get_path(&mut machine_state)
                    .append(&mut added_rail);
                machine_modes.push(GoAroundMachine::LastLeg);
            }
            GoAroundMachine::LastLeg => {
                trace!(
                    "[LastLeg] making straight {}",
                    machine_mode.get_steps(&machine_state) + 1
                );

                let mut added_rail: Vec<Rail> = Vec::new();

                machine_mode.incriment_steps(&mut machine_state);
                for step in 0..machine_mode.get_steps(&machine_state) {
                    let leg_straight = added_rail
                        .last()
                        .unwrap_or_else(|| machine_mode.get_previous_path_end(&machine_state))
                        .move_forward_step()
                        .into_buildable(surface, search_area, path, end);
                    match leg_straight {
                        Some(leg_straight) => {
                            added_rail.push(leg_straight);
                        }
                        None => {
                            // can't go across, need first leg to go up one more
                            let pop_accross = machine_modes.ends_with(&[GoAroundMachine::LastLeg]);
                            if pop_accross {
                                machine_modes.pop();
                            }
                            trace!("[ThirdLeg] [Straight{}] failed, move GoAround back", step);
                            continue 'machine;
                        }
                    }
                }

                if !machine_modes.ends_with(&[GoAroundMachine::LastLeg]) {
                    machine_modes.push(GoAroundMachine::LastLeg);
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

                machine_mode.get_path(&mut machine_state).clear();
                machine_mode
                    .get_path(&mut machine_state)
                    .append(&mut added_rail);
                // info!(
                //     "end of stack first {} second {} third {}",
                //     first_leg_steps, across_steps, last_leg_steps
                // );
                info!("end of stack");
                break;
            }
        }
    }

    for state in machine_state {
        for leg_path in state.path {
            path.push(RailPointCompare::new(leg_path));
        }
    }
}

fn replace_end_of_slice<T>(slice: &mut [T], new_value: T) {
    assert!(!slice.is_empty(), "slice is empty");
    slice[slice.len() - 1] = new_value;
}
