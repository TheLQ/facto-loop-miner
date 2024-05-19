use crate::navigator::mori::{Rail, RailDirection, RailMode, TurnType};
use crate::navigator::rail_point_compare::RailPointCompare;
use crate::surface::surface::Surface;
use crate::surfacev::varea::VArea;
use crate::surfacev::vpoint::VPoint;
use crate::surfacev::vsurface::VSurface;
use crossbeam::queue::SegQueue;
use itertools::Itertools;
use opencv::core::add;
use std::cmp::max;
use std::path::Path;
use strum::Display;
use tracing::{error, info, trace};

/// Pathfinder v2, Josuiji Shinri - Shinkansen
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
    info!("----- Shinri start {:?} to {:?}", start, end);

    let test_x_success = |p: &Rail| p.endpoint.x() >= end.endpoint.x();
    let test_y_success = |p: &Rail| p.endpoint.y() >= end.endpoint.y();

    let mut state_stack = vec![ShinriKansen {
        kind: ShinriKansenKind::GoStraight,
        path: Vec::new(),
        steps: None,
    }];

    loop {
        let value = state_stack
            .last_mut()
            .unwrap()
            .navigate(surface, search_area, &start);
        if test_x_success(state_stack.last().unwrap().path.last().unwrap()) {
            break;
        } else {
            let mut next = ShinriKansen {
                kind: ShinriKansenKind::GoAroundAway,
                steps: None,
                path: Vec::new(),
            };
            let res = next.navigate(
                surface,
                search_area,
                state_stack.last().unwrap().last_rail(),
            );
            state_stack.push(next);
            break;
        }
    }

    Some(state_stack.into_iter().map(|v| v.path).flatten().collect())
}

pub fn shinri_start_2(
    surface: &VSurface,
    start: Rail,
    end: Rail,
    search_area: &VArea,
) -> Option<Vec<Rail>> {
    info!("----- Shinri start {:?} to {:?}", start, end);
    let mut path = vec![RailPointCompare::new(start)];

    let test_x_success = |p: &RailPointCompare| p.inner.endpoint.x() >= end.endpoint.x();
    let test_y_success = |p: &RailPointCompare| p.inner.endpoint.y() >= end.endpoint.y();

    let nav_result = navigate_axis_until(surface, &mut path, &test_x_success, search_area, &end);
    match nav_result {
        Ok(()) => {}
        Err(NavigateAxisErr::Crashed) => {
            error!("EARLY!");
            return Some(path.into_iter().map(|c| c.inner).collect());
        }
    }
    // path.pop().unwrap_or_else(|v| );
    if path.pop().is_none() {
        panic!("why??? {}", path.len());
    }

    let the_turn = path.last().unwrap().clone().inner.move_left();
    let Some(the_turn) = the_turn.into_buildable(surface, search_area, &path, &end) else {
        panic!("todo")
    };
    path.push(RailPointCompare::new(the_turn));
    info!("the turn >>>>>>>>>");

    navigate_axis_until(surface, &mut path, &test_y_success, search_area, &end);

    // if test_x_success(&path.last().unwrap()) {}

    Some(path.into_iter().map(|c| c.inner).collect())
}

enum NavigateAxisErr {
    Crashed,
}

fn navigate_axis_until<FS>(
    surface: &VSurface,
    path: &mut Vec<RailPointCompare>,
    test_navigate_success: &FS,
    search_area: &VArea,
    end: &Rail,
) -> Result<(), NavigateAxisErr>
where
    FS: Fn(&RailPointCompare) -> bool,
{
    loop {
        trace!("starting axis");

        go_straight_until(surface, path, test_navigate_success, search_area, end);
        if test_navigate_success(path.last().unwrap()) {
            trace!("navigated axis!");
            break;
        }

        let pre_around_length = path.len();
        go_navigate_around(surface, path, test_navigate_success, search_area, end)
            .map_err(|_| NavigateAxisErr::Crashed)?;

        if test_navigate_success(path.last().unwrap()) {
            error!("no don't want this!");
            break;
        }
    }

    // match straight_result {
    //     StraightResult::NeedGoAround =>
    //
    // }

    Ok(())
}

fn go_straight_until<FS>(
    surface: &VSurface,
    path: &mut Vec<RailPointCompare>,
    test_navigate_success: &FS,
    search_area: &VArea,
    end: &Rail,
) -> GoStraightResult
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
            return GoStraightResult::NeedGoAround;
        };

        let next_compare = RailPointCompare::new(next);
        let should_stop = test_navigate_success(&next_compare);

        trace!("advancing {:?}", next_compare.inner);
        path.push(next_compare);

        if should_stop {
            return GoStraightResult::NavigateSuccess;
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

enum GoAroundErr {
    CrashedAtTurn,
}

/// Go perpendicular to axis, across, then back
fn go_navigate_around<FS>(
    surface: &VSurface,
    path: &mut Vec<RailPointCompare>,
    test_navigate_success: FS,
    search_area: &VArea,
    end: &Rail,
) -> Result<(), GoAroundErr>
where
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
                    let last = path.pop().unwrap();
                    if last.inner.mode != RailMode::Straight {
                        return Err(GoAroundErr::CrashedAtTurn);
                    }
                    // assert_eq!(
                    //     last.inner.mode,
                    //     RailMode::Straight,
                    //     "woah! {:?}",
                    //     last.inner
                    // );
                    pop_existing = false;
                }
                let axis_rail = &path.last().ok_or(GoAroundErr::CrashedAtTurn)?.inner;

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

    Ok(())
}

fn replace_end_of_slice<T>(slice: &mut [T], new_value: T) {
    assert!(!slice.is_empty(), "slice is empty");
    slice[slice.len() - 1] = new_value;
}

struct ShinriLeg {
    steps: u32,
    turn: TurnType,
}

impl ShinriLeg {
    fn try_advance() {}
}

enum TryAdvanceResult {}

// struct ShinriKansen {
//     legs: Vec<ShinriLeg>,
// }

enum GoStraightResult {
    NeedGoAround,
    NavigateSuccess,
}

enum NavigateResult {
    Done,
    DecrementPrevious,
}

struct ShinriKansen {
    path: Vec<Rail>,
    steps: Option<u32>,
    kind: ShinriKansenKind,
}

enum ShinriKansenKind {
    GoStraight,
    GoAroundAway,
    GoAroundStraight,
    GoAroundBack,
}

impl ShinriKansen {
    #[must_use]
    pub fn navigate(
        &mut self,
        surface: &VSurface,
        search_area: &VArea,
        begin: &Rail,
    ) -> NavigateResult {
        self.path.clear();
        match self.kind {
            ShinriKansenKind::GoStraight | ShinriKansenKind::GoAroundStraight => {
                let move_result = inner_move_forward_loop(
                    surface,
                    search_area,
                    Some(begin),
                    self.steps,
                    &mut self.path,
                );
                match move_result {
                    InnerMoveForwardResult::Success(steps) => {
                        self.steps = Some(steps);
                        return NavigateResult::Done;
                    }
                    InnerMoveForwardResult::NotEnoughSteps => panic!("????"),
                    InnerMoveForwardResult::NoSteps => panic!("????"),
                };
            }
            ShinriKansenKind::GoAroundAway => {
                inner_move_around(surface, search_area, begin, self, TurnType::Turn90)
            }
            ShinriKansenKind::GoAroundBack => {
                inner_move_around(surface, search_area, begin, self, TurnType::Turn270)
            }
        }
    }

    pub fn last_rail(&self) -> &Rail {
        self.path.last().unwrap()
    }
}

enum InnerMoveForwardResult {
    NoSteps,
    NotEnoughSteps,
    Success(u32),
}

#[must_use]
fn inner_move_forward_loop(
    surface: &VSurface,
    search_area: &VArea,
    begin: Option<&Rail>,
    to_steps: Option<u32>,
    result: &mut Vec<Rail>,
) -> InnerMoveForwardResult {
    let mut last_step = 0;
    for step in 0..to_steps.unwrap_or(u32::MAX) {
        let leg_straight = result
            .last()
            .unwrap_or_else(|| begin.unwrap())
            .move_forward_step()
            .into_buildable_simple(surface, search_area);
        match leg_straight {
            Some(leg_straight) => {
                result.push(leg_straight);
            }
            None => {
                trace!("[Straight] max {} failed", step);
                break;
            }
        }
        last_step = step;
    }

    if let Some(max_steps) = to_steps {
        if max_steps != last_step {
            return InnerMoveForwardResult::NotEnoughSteps;
        }
    }

    if result.is_empty() {
        InnerMoveForwardResult::NoSteps
    } else {
        InnerMoveForwardResult::Success(last_step + 1)
    }
}

fn inner_move_around(
    surface: &VSurface,
    search_area: &VArea,
    begin: &Rail,
    kansen: &mut ShinriKansen,
    turn_type: TurnType,
) -> NavigateResult {
    let next = begin
        .move_rotating(TurnType::Turn90)
        .into_buildable_simple(surface, search_area)
        .map(|next| kansen.path.push(next));

    let next = match next {
        None => return NavigateResult::DecrementPrevious,
        Some(first) => {
            inner_move_forward_loop(surface, search_area, None, kansen.steps, &mut kansen.path)
        }
    };

    let next = match next {
        InnerMoveForwardResult::Success(steps) => {
            kansen.steps = Some(steps);
            kansen
                .path
                .last()
                .unwrap()
                .move_rotating(turn_type.swap())
                .into_buildable_simple(surface, search_area)
        }
        InnerMoveForwardResult::NotEnoughSteps => return NavigateResult::DecrementPrevious,
        InnerMoveForwardResult::NoSteps => panic!("????"),
    };

    match next {
        Some(next) => {
            kansen.path.push(next);
            return NavigateResult::Done;
        }
        None => return NavigateResult::DecrementPrevious,
    }
}
