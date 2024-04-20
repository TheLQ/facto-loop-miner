use crate::admiral::generators::rail90::{
    dual_rail_east, dual_rail_empty_index_to_xy, dual_rail_north, dual_rail_north_empty,
    dual_rail_south, dual_rail_south_empty, dual_rail_west, dual_rail_west_empty,
    rail_degrees_east, rail_degrees_north, rail_degrees_west,
};
use crate::admiral::lua_command::fac_surface_create_entity::FacSurfaceCreateEntity;
use crate::admiral::lua_command::LuaCommand;
use crate::navigator::mori_cost::calculate_cost_for_point;
use crate::navigator::rail_point_compare::RailPointCompare;
use crate::navigator::resource_cloud::ResourceCloud;
use crate::simd_diff::SurfaceDiff;
use crate::surface::pixel::Pixel;
use crate::surface::surface::{PointU32, Surface};
use crate::surfacev::err::VResult;
use crate::surfacev::varea::VArea;
use crate::surfacev::vpoint::VPoint;
use crate::surfacev::vsurface::VSurface;
use crate::util::duration::BasicWatch;
use crate::LOCALE;
use crossbeam::atomic::AtomicCell;
use itertools::Itertools;
use num_format::Locale::pa;
use num_format::ToFormattedString;
use pathfinding::prelude::astar_mori;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use strum::AsRefStr;
use tracing::{debug, error, warn};

/// Pathfinder v1, Mori Calliope
///
/// astar powered pathfinding.
///
/// Makes a dual rail + spacing, +6 straight or 90 degree turning, path of rail from start to end.
/// Without collisions into any point on the Surface.
pub fn mori_start(
    surface: &mut VSurface,
    start: Rail,
    end: Rail,
    search_area: &VArea,
) -> Option<Vec<Rail>> {
    let pathfind_watch = BasicWatch::start();

    start.endpoint.assert_odd_8x8_position();
    // for end in ends {
    //     end.endpoint.assert_odd_8x8_position();
    // }
    end.endpoint.assert_odd_position();

    // TODO: Benchmark this vs Vec (old version),
    // let mut valid_destinations: FxHashSet<Rail> = FxHashSet::new();
    // // let step = 3;
    // for width in -RAIL_STEP_SIZE_I32..RAIL_STEP_SIZE_I32 {
    //     for height in -RAIL_STEP_SIZE_I32..RAIL_STEP_SIZE_I32 {
    //         let mut next = end.clone();
    //         next.endpoint = next.endpoint.move_xy(width, height);
    //         // surface
    //         //     .set_pixel(next.endpoint, Pixel::Highlighter)
    //         //     .unwrap();
    //         valid_destinations.insert(next);
    //     }
    // }

    let resource_cloud = ResourceCloud::from_surface(surface);

    // TODO let mut working_buffer = surface.surface_diff();
    let working_buffer = SurfaceDiff::TODO_new();

    let metric_successors = AtomicU64::new(1);
    let metric_start = AtomicCell::new(Instant::now());

    // let end_points: Vec<VPoint> = ends.iter().map(|r| r.endpoint).collect();
    // let end = VArea::from_arbitrary_points(&end_points).point_center();
    // let end = end_points[0];
    debug!("Mori start {:?} end {:?}", start, end);
    // Forked function adds parents and cost params to each successor call. Used for limits
    let start_compare = RailPointCompare::new(start);
    let pathfind = astar_mori(
        &start_compare,
        |(_rail, parents, _total_cost)| {
            let (rail, parents) = parents.split_last().unwrap();
            rail.inner.successors(
                parents,
                surface,
                &start_compare.inner,
                &end,
                &resource_cloud,
                search_area,
                &metric_successors,
                &metric_start,
            )
        },
        |_p| 1,
        |p| end == p.inner,
    );
    // let pathfind = threaded_searcher::<Rail, _, _>(
    //     start.clone(),
    //     |parents| {
    //         let (rail, parents) = parents.split_last().unwrap();
    //         rail.successors(
    //             parents,
    //             surface,
    //             &end,
    //             &resource_cloud,
    //             &search_area,
    //             &metric_successors,
    //             &metric_start,
    //         )
    //     },
    //     |p| valid_destinations.contains(p),
    // );
    let mut result = None;
    if let Some(pathfind) = pathfind {
        let (path, path_cost) = pathfind;
        debug!("built path {} long with {}", path.len(), path_cost);

        result = Some(path.into_iter().map(|c| c.inner).collect());
    } else {
        warn!(
            "failed to pathfind from {:?} to {:?}",
            start_compare.inner, end
        );
    }

    debug!("+++ Mori finished in {}", pathfind_watch,);

    // unsafe {
    //     debug!(
    //         "metric successors called {}",
    //         METRIC_SUCCESSORS.to_formatted_string(&LOCALE),
    //     );
    //     METRIC_SUCCESSORS = 0;
    // }
    // METRIC_SUCCESSOR.swap(0, Ordering::Relaxed);

    result
}

// #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
// pub struct RailPoint {
//     pub(crate) x: i32,
//     pub(crate) y: i32,
// }

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum RailMode {
    Straight,
    Turn(TurnType),
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum TurnType {
    Turn90,
    Turn270,
}

impl TurnType {
    pub fn rotations(&self) -> usize {
        match self {
            TurnType::Turn90 => 1,
            TurnType::Turn270 => 3,
        }
    }

    pub fn swap(&self) -> Self {
        match self {
            TurnType::Turn90 => TurnType::Turn270,
            TurnType::Turn270 => TurnType::Turn90,
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize, AsRefStr)]
pub enum RailDirection {
    Up,
    Down,
    Left,
    Right,
}

impl RailDirection {
    pub fn spinner(&self, post_rotations: usize) -> RailDirection {
        let mut directions = RAIL_DIRECTION_CLOCKWISE.iter().cycle();

        while directions.next().unwrap() != self {}

        let mut new_direction = directions.next().unwrap();
        for _ in 1..post_rotations {
            new_direction = directions.next().unwrap();
        }
        new_direction.clone()
    }

    pub fn is_same_axis(&self, other: &Self) -> bool {
        match self {
            RailDirection::Up | RailDirection::Down => {
                *other == RailDirection::Up || *other == RailDirection::Down
            }
            RailDirection::Left | RailDirection::Right => {
                *other == RailDirection::Left || *other == RailDirection::Right
            }
        }
    }

    pub fn to_factorio(&self) -> &'static str {
        match self {
            RailDirection::Up => "north",
            RailDirection::Down => "south",
            RailDirection::Left => "west",
            RailDirection::Right => "east",
        }
    }
}

const RAIL_DIRECTION_CLOCKWISE: [RailDirection; 4] = [
    RailDirection::Up,
    RailDirection::Right,
    RailDirection::Down,
    RailDirection::Left,
];

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Rail {
    pub endpoint: VPoint,
    pub direction: RailDirection,
    pub mode: RailMode,
}

const DUAL_RAIL_SIZE: u32 = 3;

// static METRIC_SUCCESSOR: AtomicU64 = AtomicU64::new(0);
// // static mut METRIC_SUCCESSORS: u64 = 0;
// static mut METRIC_SUCCESSOR_START: Instant = lazy_static::Instant::now();

pub const RAIL_STEP_SIZE: u32 = 8;
pub const RAIL_STEP_SIZE_I32: i32 = RAIL_STEP_SIZE as i32;

impl Rail {
    pub fn new_straight(endpoint: VPoint, direction: RailDirection) -> Self {
        let res = Rail {
            endpoint,
            mode: RailMode::Straight,
            direction,
        };

        res
    }

    /// XY positions underneath this section of rail
    fn area(&self) -> Vec<VPoint> {
        let mut res = Vec::new();
        match &self.mode {
            RailMode::Straight => {
                for rail in &self.build_dual_straight_behind_rail(RAIL_STEP_SIZE) {
                    res.extend_from_slice(&rail.endpoint.get_entity_area_2x2());
                }
            }
            RailMode::Turn(turn_type) => {
                // ↓↓↓↓↓↓↓↓↓ OK-ish FIX
                let leg = self.clone();
                for rail in leg.build_dual_straight_behind_rail(RAIL_STEP_SIZE) {
                    res.extend_from_slice(&rail.endpoint.get_entity_area_2x2());
                }

                let leg = self
                    .move_force_rotate_clockwise(2)
                    .move_forward_step()
                    .move_force_rotate_clockwise(turn_type.rotations());
                for rail in leg.build_dual_straight_behind_rail(RAIL_STEP_SIZE) {
                    res.extend_from_slice(&rail.endpoint.get_entity_area_2x2());
                }
                // ↑↑↑↑↑↑↑↑↑ OK-ish FIX

                // // ↓↓↓↓↓↓↓↓↓ GHETTO FIX
                // let mut last_leg = self.clone();
                // last_leg.mode = RailMode::Straight;
                // res.append(&mut last_leg.area());
                //
                // last_leg = last_leg
                //     .move_force_rotate_clockwise(2)
                //     .move_forward_step()
                //     .move_force_rotate_clockwise(2);
                // res.append(&mut last_leg.area());
                //
                // if turn_type == &TurnType::Turn90 {
                //     let mut corner_cover = self.clone();
                //     corner_cover.mode = RailMode::Straight;
                //     corner_cover.move_force_forward_single_num_mut(-3);
                //     res.extend_from_slice(&corner_cover.area());
                //     // this lazy way heavily overlaps last_leg
                //     res.sort();
                //     res.dedup();
                // }
                //
                // let first_leg = self.move_force_rotate_clockwise(2).move_forward_step();
                // let first_leg = first_leg.move_force_rotate_clockwise(match &turn_type {
                //     TurnType::Turn90 => 1,
                //     TurnType::Turn270 => 3,
                // });
                // res.append(&mut first_leg.area());
                //
                // let first_leg = first_leg.move_forward_step();
                // res.append(&mut first_leg.area());
                //
                // let before = res.len();
                // res.sort();
                // res.dedup();
                // let after = res.len();
                // // if ![8, 0].contains(&(before - after)) {
                // //     panic!("Asd {} {} {}", before, after, before - after);
                // // }
                //
                // // ↑↑↑↑↑↑↑↑↑ GHETTO FIX

                // ----- PROPER FIX
                // let endpoint = self.endpoint;
                //
                // let (template, x_sign, y_sign) = match (&self.direction, turn_type) {
                //     (RailDirection::Left, TurnType::Turn90) => {
                //         (rail_turn_template_down_right(), 1, 1)
                //     }
                //     (RailDirection::Right, TurnType::Turn270) => {
                //         (rail_turn_template_up_left(), -1, 1)
                //     }
                //     (direction, turn) => {
                //         unimplemented!("no template for {:?} {:?}", direction, turn)
                //     }
                // };
                //
                // for (y_pos, row) in template.iter().enumerate() {
                //     for (x_pos, value) in row.iter().enumerate() {
                //         if *value {
                //             let this_point =
                //                 endpoint.move_xy(x_pos as i32 * x_sign, y_pos as i32 * y_sign);
                //             res.push(this_point);
                //         }
                //     }
                // }
                // ----- PROPER FIX

                // // TODO: Temporary solution

                // ↓↓↓↓↓↓↓↓↓ GRID
                // let (grid, top_right_corner_rail) = match (&self.direction, turn_type) {
                //     (RailDirection::Right, TurnType::Turn270) => (
                //         dual_rail_west_empty(),
                //         self.move_force_rotate_clockwise(2).move_forward_step(),
                //     ),
                //     (RailDirection::Left, TurnType::Turn90) => (
                //         dual_rail_north_empty(),
                //         // self.move_force_rotate_clockwise(2)
                //         //     .move_forward_step()
                //         //     .move_force_rotate_clockwise(3)
                //         //     .move_forward_step(),
                //         self.clone(),
                //     ),
                //     (direction, turn) => {
                //         unimplemented!("no template for {:?} {:?}", direction, turn)
                //     }
                // };
                // let grid_bool = grid.into_inner();
                // for (i, is_empty) in grid_bool.iter().enumerate() {
                //     if !is_empty {
                //         let not_empty_point = top_right_corner_rail.endpoint
                //             + dual_rail_empty_index_to_xy(&grid_bool, i);
                //         println!("drawing {:?}", not_empty_point);
                //         res.push(not_empty_point);
                //     }
                // }
                // ↑↑↑↑↑↑↑↑↑ GRID

                // let next = self.move_force_rotate_clockwise(2);
                // let next = next.move_forward_step();
                // let next_last_leg = next.move_force_rotate_clockwise(2);
                // res.extend_from_slice(&next_last_leg.endpoint.get_entity_area_2x2());

                // let next = self.move_force_rotate_clockwise(1);
                // let next = next.move_forward_step();
                // res.extend_from_slice(&next.area());
                // res.extend_from_slice(&next.endpoint.get_entity_area_2x2());

                // let source_rail = Rail {
                //     endpoint: source_point.clone(),
                //     direction: source_direction.clone(),
                //     mode: RailMode::Straight,
                // };
                // // debug!("source {:?}", source_rail);
                //
                // let first_leg = source_rail.move_forward_step();
                // // debug!("first_leg {:?}", first_leg);
                //
                // // let is_left_turn =
                // //     source_rail.move_force_rotate_clockwise(1).direction == self.direction;
                // // if is_left_turn {
                // //     let dog_leg = first_leg.move_force_forward_steps(1);
                // //     res.extend(dog_leg.area());
                // //     // debug!("dog_leg {:?}", dog_leg);
                // // }
                //
                // let mut second_leg = first_leg.clone();
                // second_leg.direction = self.direction.clone();
                // second_leg = second_leg.move_forward();
                // // debug!("second_leg {:?}", second_leg);
                //
                // res.extend(first_leg.area());
                //
                // res.extend(second_leg.area());
                //
                // // very first row is the source's
                // unimplemented!("asd");
                // // for width in 1..RAIL_STEP_SIZE {
                // //     for height in 0..RAIL_STEP_SIZE {
                // //         let mut next = source_rail.clone();
                // //         next.move_force_forward_mut(width);
                // //         next.direction = self.direction.clone();
                // //         next.move_force_forward_mut(height);
                // //
                // //         res.push(next.endpoint);
                // //     }
                // // }
            }
        }

        // for entry in &res {
        //     debug!("ENTRY {:?}", entry);
        // }
        // let before = res.len();
        // res.sort();
        // res.dedup();
        // let after = res.len();
        // if before != after {
        //     warn!(
        //         "reduced area duplicates from {} to {} for {:?}",
        //         before, after, self
        //     );
        // }

        res
    }

    pub fn to_factorio_entities(&self, result: &mut Vec<Box<dyn LuaCommand>>) {
        self.endpoint.assert_odd_position();
        match &self.mode {
            RailMode::Straight => {
                for rail in self.build_dual_straight_behind_rail(RAIL_STEP_SIZE) {
                    rail.to_facto_entity(result);
                }
            }
            RailMode::Turn(turn_type) => {
                // ending with 2x rails
                let point = self.clone();
                point
                    .move_force_rotate_clockwise(1)
                    .move_forward_single_num(1)
                    .move_force_rotate_clockwise(1)
                    .to_facto_entities_line(result, 0, 2);
                point
                    .move_force_rotate_clockwise(3)
                    .move_forward_single_num(1)
                    .move_force_rotate_clockwise(3)
                    .to_facto_entities_line(result, 0, 2);

                // starting with 2x rails
                let point = self
                    .move_force_rotate_clockwise(2)
                    .move_forward_step()
                    .move_force_rotate_clockwise(turn_type.swap().rotations())
                    .move_forward_step();
                point
                    .move_force_rotate_clockwise(1)
                    .move_forward_single_num(1)
                    .move_force_rotate_clockwise(1)
                    .to_facto_entities_line(result, 1, 2);
                point
                    .move_force_rotate_clockwise(3)
                    .move_forward_single_num(1)
                    .move_force_rotate_clockwise(3)
                    .to_facto_entities_line(result, 1, 2);

                const OFFSET_8X_RAIL: i32 = RAIL_STEP_SIZE_I32 * 2;
                const OFFSET_8X_RAIL_2: i32 = OFFSET_8X_RAIL + 2;
                const OFFSET_8X_RAIL_4: i32 = OFFSET_8X_RAIL - 4;
                match (&self.direction, turn_type) {
                    // Straight Down Turn Left
                    (RailDirection::Left, TurnType::Turn90) => {
                        dual_rail_north(self.endpoint.move_xy(4, -2), result);
                    }
                    // Straight Down Turn Right
                    (RailDirection::Right, TurnType::Turn270) => {
                        dual_rail_west(self.endpoint.move_xy(-OFFSET_8X_RAIL_2, -2), result);
                    }
                    // Straight Up Turn Left
                    (RailDirection::Left, TurnType::Turn270) => {
                        dual_rail_east(self.endpoint.move_xy(4, -OFFSET_8X_RAIL_4), result);
                    }
                    // Straight Up Turn Right
                    (RailDirection::Right, TurnType::Turn90) => {
                        dual_rail_south(
                            self.endpoint.move_xy(-OFFSET_8X_RAIL_2, -OFFSET_8X_RAIL_4),
                            result,
                        );
                    }
                    // Straight Left Turn Left
                    (RailDirection::Down, TurnType::Turn270) => {
                        dual_rail_south(self.endpoint.move_xy(-2, 4), result);
                    }
                    // Straight Left Turn Right
                    (RailDirection::Up, TurnType::Turn90) => {
                        dual_rail_west(self.endpoint.move_xy(-2, -OFFSET_8X_RAIL_2), result);
                    }
                    // Straight Right Turn Left
                    (RailDirection::Up, TurnType::Turn270) => {
                        dual_rail_north(
                            self.endpoint.move_xy(-OFFSET_8X_RAIL_4, -OFFSET_8X_RAIL_2),
                            result,
                        );
                    }
                    // Straight Right Turn Right
                    (RailDirection::Down, TurnType::Turn90) => {
                        dual_rail_east(self.endpoint.move_xy(-OFFSET_8X_RAIL_4, 4), result);
                    }
                };
            }
        }
    }

    pub fn build_dual_straight_behind_rail(&self, length: u32) -> Vec<Rail> {
        let mut result = Vec::with_capacity(length as usize * 2);

        self.move_force_rotate_clockwise(1)
            .move_forward_single_num(1)
            .move_force_rotate_clockwise(1)
            .build_straight_rail_line(&mut result, 0, length);

        self.move_force_rotate_clockwise(3)
            .move_forward_single_num(1)
            .move_force_rotate_clockwise(3)
            .build_straight_rail_line(&mut result, 0, length);

        result
    }

    fn build_straight_rail_line(&self, result: &mut Vec<Rail>, start: u32, end: u32) {
        for i in start..end {
            let next = self.move_forward_single_num(i);
            result.push(next);
        }
    }

    fn to_facto_entities_line(&self, result: &mut Vec<Box<dyn LuaCommand>>, start: u32, end: u32) {
        for i in start..end {
            let next = self.move_forward_single_num(i);
            next.to_facto_entity(result);
        }
    }

    fn to_facto_entity(&self, result: &mut Vec<Box<dyn LuaCommand>>) {
        result.push(
            FacSurfaceCreateEntity::new_rail_straight(
                self.endpoint.to_f32(),
                self.direction.clone(),
            )
            .into_boxed(),
        );
    }

    pub fn distance_to(&self, other: &Rail) -> u32 {
        let a = &self.endpoint;
        let b = &other.endpoint;
        a.distance_to(b)
    }

    pub fn move_force_rotate_clockwise(&self, rotations: usize) -> Self {
        let mut next = self.clone();
        next.direction = next.direction.spinner(rotations);
        next
    }

    // pub fn move_backwards_toward_water(
    //     &self,
    //     surface: &VSurface,
    //     working_buffer: &mut SurfaceDiff,
    // ) -> Self {
    //     // come in as far away as possible
    //     // optimize area around base
    //     let mut next = self.clone();
    //     next = next.move_force_rotate_clockwise(2);
    //     // TODO
    //     loop {
    //         let next_rail = next.move_forward();
    //         if let Some(next_rail) = next_rail.into_buildable(surface, working_buffer) {
    //             next = next_rail;
    //         } else {
    //             break;
    //         }
    //     }
    //     // point back in original direction
    //     next = next.move_force_rotate_clockwise(2);
    //     // back away from water with a full step
    //     next = next.move_forward();
    //     // full step again for spacing (eg in a cove)
    //     next = next.move_forward();
    //
    //     debug!("move backwards from {:?} to {:?}", self, next);
    //
    //     next
    // }

    pub fn move_forward_step(&self) -> Self {
        self.move_forward_step_num(1)
    }

    pub fn move_forward_step_num(&self, steps: u32) -> Self {
        let mut next = self.move_forward_single_num(RAIL_STEP_SIZE * steps);
        next.mode = RailMode::Straight;
        next
    }

    pub fn move_forward_single(&self) -> Self {
        let mut next = self.move_forward_single_num(1);
        next.mode = RailMode::Straight;
        next
    }

    pub fn move_forward_single_num(&self, steps: u32) -> Self {
        let mut next = self.clone();
        next.move_force_forward_single_num_mut(steps as i32);
        next
    }

    fn move_force_forward_single_num_mut(&mut self, steps: i32) {
        // rail is 2x2
        const RAIL_ENTITY_SIZE: i32 = 2;
        let steps = steps * RAIL_ENTITY_SIZE;
        self.endpoint = match self.direction {
            RailDirection::Up => self.endpoint.move_y(steps),
            RailDirection::Down => self.endpoint.move_y(-steps),
            RailDirection::Left => self.endpoint.move_x(-steps),
            RailDirection::Right => self.endpoint.move_x(steps),
        };
    }

    /// may be negative
    // fn distance_in_direction_to_point(&self, end: &PointU32) -> i32 {
    //     match self.direction {
    //         RailDirection::Up => end.y as i32 - self.endpoint.y,
    //         RailDirection::Down => -(end.y as i32 - self.endpoint.y),
    //         RailDirection::Left => end.x as i32 - self.endpoint.x,
    //         RailDirection::Right => -(end.x as i32 - self.endpoint.x),
    //     }
    // }

    pub fn move_left(&self) -> Self {
        self.move_rotating(TurnType::Turn270)
    }

    pub fn move_right(&self) -> Self {
        self.move_rotating(TurnType::Turn90)
    }

    fn move_rotating(&self, turn_type: TurnType) -> Self {
        let next = self.clone();
        let next = next.move_forward_step();
        // let next = next.move_forward_step();
        let next = next.move_force_rotate_clockwise(turn_type.rotations());
        // let next = next.move_forward_step();
        let mut next = next.move_forward_step();
        next.mode = RailMode::Turn(turn_type);
        next
    }

    fn successors(
        &self,
        parents_compare: &[RailPointCompare],
        surface: &VSurface,
        start: &Rail,
        end: &Rail,
        // ends: &[Rail],
        resource_cloud: &ResourceCloud,
        // working_buffer: &mut SurfaceDiff,
        search_area: &VArea,
        metric_successors: &AtomicU64,
        metric_start: &AtomicCell<Instant>,
    ) -> Vec<(RailPointCompare, u32)> {
        let metric_successors = metric_successors.fetch_add(1, Ordering::Relaxed);
        if metric_successors % 100_000 == 0 {
            let now = Instant::now();
            // let metric_start = metric_start.swap(now);
            // let rate_paths_per_second = 1_000_000.0 / since_last;
            let metric_start = metric_start.load();
            let since_last = (now - metric_start).as_secs().max(1) as f32;
            let rate_paths_per_second = metric_successors as f32 / since_last;
            debug!(
                "successor {} spot parents {} in {} paths/second total {} seconds",
                metric_successors.to_formatted_string(&LOCALE),
                parents_compare.len(),
                (rate_paths_per_second as u32).to_formatted_string(&LOCALE),
                since_last,
            );
        }

        // if parents.len() > 800 {
        //     return Vec::new();
        // }
        // self.endpoint.assert_odd_8x8_position();
        // debug!("testing {:?}", self);

        if parents_compare.iter().any(|p| p.inner == *self) {
            panic!("crashing found myself!");
        }

        let mut working_buffer_owned = SurfaceDiff::TODO_new();
        let working_buffer = &mut working_buffer_owned;

        let mut res = Vec::new();
        if let Some(rail) =
            self.move_forward_step()
                .into_buildable(surface, search_area, parents_compare, end)
        {
            let cost = calculate_cost_for_point(
                &rail,
                start,
                &end.endpoint,
                resource_cloud,
                parents_compare,
            );
            res.push((RailPointCompare::new(rail), cost));
        }

        if let Some(rail) =
            self.move_left()
                .into_buildable(surface, search_area, parents_compare, end)
        {
            let cost = calculate_cost_for_point(
                &rail,
                start,
                &end.endpoint,
                resource_cloud,
                parents_compare,
            );
            res.push((RailPointCompare::new(rail), cost));
        }
        if let Some(rail) =
            self.move_right()
                .into_buildable(surface, search_area, parents_compare, end)
        {
            let cost = calculate_cost_for_point(
                &rail,
                start,
                &end.endpoint,
                resource_cloud,
                parents_compare,
            );
            res.push((RailPointCompare::new(rail), cost))
        }
        // debug!(
        //     "for {:?} found {}",
        //     &self,
        //     res.iter().map(|(rail, _)| format!("{:?}", rail)).join("|")
        // );
        res
    }

    pub fn into_buildable(
        self,
        surface: &VSurface,
        search_area: &VArea,
        parents_compare: &[RailPointCompare],
        end: &Rail,
    ) -> Option<Self> {
        self.endpoint.assert_odd_8x8_position();
        if !search_area.contains_point(&self.endpoint) {
            return None;
        }

        // if let Some(found_goal) = ends.iter().find(|e| e.endpoint == self.endpoint) {
        //     if *found_goal != self {
        //         return None;
        //     }
        // }
        // if end.endpoint == self.endpoint && end == &self {
        //     return None;
        // }

        // anti-recursion
        // if parents_compare
        //     .iter()
        //     .filter(|p| p.inner.endpoint == self.endpoint)
        //     .count()
        //     > 1
        // {
        //     // return None;
        //     error!("current {:?}", self);
        //     for parent in parents_compare {
        //         error!("parent {:?}", parent.inner);
        //     }
        //     panic!("shouldn't happen anymore?");
        // }
        // extra anti-recursion
        // let mut set: HashSet<&Rail> = HashSet::new();
        // set.insert(&self);
        // for parent in parents {
        //     if set.contains(parent) {
        //         return None;
        //     }
        //     set.insert(parent);
        // }

        const SIZE: u32 = 0;
        // if self.endpoint.x() < 4000 {
        //     None
        // } else {
        match 5 {
            // 1 => self.into_buildable_sequential(surface),
            // 2 => self.into_buildable_parallel(surface),
            // 3 => self.into_buildable_avx(surface, working_buffer),
            4 => {
                if self.is_area_buildable(surface) {
                    Some(self)
                } else {
                    None
                }
            }
            5 => {
                if self.is_area_buildable_fast(surface) {
                    Some(self)
                } else {
                    None
                }
            }
            _ => panic!("0"),
        }
        // let seq = self.clone().into_buildable_sequential(surface);
        // let avx = self.clone().into_buildable_avx(surface, working_buffer);
        // match (seq, avx) {
        //     (None, None) => None,
        //     (Some(left), Some(right)) => {
        //         if left != right {
        //             panic!("UNEQUAL left {:?} right {:?}", left, right);
        //         }
        //         Some(left)
        //     }
        //     (left, right) => panic!("self {:?} left {:?} right {:?}", self, left, right),
        // }
        // }
    }

    fn is_area_buildable(&self, surface: &VSurface) -> bool {
        self.area()
            .into_iter()
            .all(|area_point| is_buildable_point_ref(surface, area_point))
    }

    fn is_area_buildable_fast(&self, surface: &VSurface) -> bool {
        surface.is_points_free(&self.area())
    }

    // fn into_buildable_sequential(self, surface: &VSurface) -> Option<Self> {
    //     if let Some(area) = self.area() {
    //         area.into_iter()
    //             .flat_map(|v| v.get_entity_area_2x2())
    //             .fold(Some(self), |acc, cur| {
    //                 acc.and_then(|acc| {
    //                     is_buildable_point_take(surface, cur)
    //                 })
    //                 // acc.and_then(|acc| {
    //                 //     cur.and_then(|v| is_buildable_point_take(surface, v).map(|_| Some(acc)))
    //                 // })
    //             })
    //             .flatten()
    //         // let points: Vec<Option<PointU32>> = area.into_iter().map(|v| filter_available_points(&v, surface)).collect();
    //         // if points.contains(None) {
    //         //     return None;
    //         // }
    //         // .reduce(|total, is_buildable| total.and(is_buildable))
    //         // .unwrap()
    //         // // .flatten()
    //         // .map(|_| self)
    //     } else {
    //         None
    //     }
    // }

    // fn into_buildable_parallel(self, surface: &Surface) -> Option<Self> {
    //     // observations: way slower than sequential, especially in --release mode
    //     if let Some(area) = self.area() {
    //         let area_buildable_opt: Vec<Option<PointU32>> = area
    //             .par_iter()
    //             .filter_map(|v| filter_buildable_points(v, surface))
    //             .flat_map_iter(|v| v)
    //             .map(|game_pont| is_buildable_point_u32_take(surface, game_pont))
    //             .collect();
    //
    //         area_buildable_opt
    //             .into_iter()
    //             .reduce(|total, is_buildable| total.and(is_buildable))
    //             .unwrap()
    //             .map(|_| self)
    //     } else {
    //         None
    //     }
    // }

    // fn into_buildable_avx(
    //     self,
    //     surface: &Surface,
    //     working_buffer: &mut SurfaceDiff,
    // ) -> Option<Self> {
    //     if let Some(area) = self.area() {
    //         if let Some(points) = area
    //             .into_iter()
    //             .flat_map(|v| get_rail_area_points(&v, surface))
    //             .map(|v| v.map(|v| surface.xy_to_index_point_u32(v)))
    //             .try_collect()
    //         {
    //             if working_buffer.is_positions_free(points) {
    //                 return Some(self);
    //             }
    //         }
    //     }
    //     None
    // }

    pub fn distance_between_parallel_axis(&self, other: &Rail) -> i32 {
        match self.direction {
            RailDirection::Left | RailDirection::Right => self.endpoint.subtract_x(&other.endpoint),
            RailDirection::Up | RailDirection::Down => self.endpoint.subtract_y(&other.endpoint),
        }
    }

    pub fn distance_between_perpendicular_axis(&self, other: &Rail) -> i32 {
        match self.direction {
            RailDirection::Left | RailDirection::Right => self.endpoint.subtract_y(&other.endpoint),
            RailDirection::Up | RailDirection::Down => self.endpoint.subtract_x(&other.endpoint),
        }
    }
}

impl Hash for Rail {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.endpoint.hash(state)
    }
}

// impl RailPoint {
//     fn round(&self) -> Self {
//         let mut next = self.clone();
//         next.x = if next.x % 2 == 0 { next.x + 1 } else { next.x };
//         next.y = if next.y % 2 == 0 { next.y + 1 } else { next.y };
//         next
//     }
//
//     fn to_game_points(&self, surface: &Surface) -> Vec<PointU32> {
//         [
//             self.clone(),
//             {
//                 let mut v = self.clone();
//                 v.x -= 1;
//                 v
//             },
//             {
//                 let mut v = self.clone();
//                 v.y -= 1;
//                 v
//             },
//             {
//                 let mut v = self.clone();
//                 v.x -= 1;
//                 v.y -= 1;
//                 v
//             },
//         ]
//         .iter()
//         .filter_map(|v| v.to_point_u32_surface(surface))
//         .collect()
//     }
//
//     fn is_negative(&self) -> bool {
//         self.x < 0 || self.y < 0
//     }
//
//     pub fn to_point_u32(&self) -> Option<PointU32> {
//         if self.is_negative() {
//             None
//         } else {
//             Some(PointU32 {
//                 x: self.x.try_into().unwrap(),
//                 y: self.y.try_into().unwrap(),
//             })
//         }
//     }
//
//     fn to_point_u32_surface(&self, surface: &Surface) -> Option<PointU32> {
//         self.to_point_u32().and_then(|p| {
//             if p.x < surface.width && p.y < surface.height {
//                 Some(p)
//             } else {
//                 None
//             }
//         })
//     }
//
//     pub fn from_point_u32(point: PointU32) -> Self {
//         RailPoint {
//             x: point.x as i32,
//             y: point.y as i32,
//         }
//     }
// }

fn is_buildable_point_ref(surface: &VSurface, point: VPoint) -> bool {
    if surface.is_point_out_of_bounds(&point) {
        return false;
    }
    match surface.get_pixel(&point) {
        Pixel::Empty => {
            // debug!("empty at {:?}", &point);
            true
        }
        _existing => {
            // debug!("blocked at {:?} by {:?}", &point, _existing);
            false
        }
    }
}

fn is_buildable_point_u32<'p>(surface: &Surface, point: &'p PointU32) -> Option<&'p PointU32> {
    if !surface.xy_in_range_point_u32(point) {
        return None;
    }
    match surface.get_pixel_point_u32(point) {
        Pixel::Empty => Some(point),
        _existing => {
            // debug!("blocked at {:?} by {:?}", &position, existing);
            None
        }
    }
}

pub fn write_rail(surface: &mut VSurface, path: &[Rail]) -> VResult<()> {
    // let special_endpoint_pixels: Vec<VPoint> = path.iter().map(|v| v.endpoint).collect();

    let mut total_rail = 0;
    for path_rail in path {
        // debug!("writing rail start at {:?}", path_rail.endpoint);
        for path_area_point in path_rail.area() {
            total_rail += 1;
            surface.set_pixel(path_area_point, Pixel::Rail)?;
            // debug!("writing rail at {:?}", path_area_point);

            // TODO: wtf??
            // let mut new_pixel = match surface.get_pixel_point_u32(&path_area_point) {
            //     Pixel::Rail => {
            //         debug!(
            //             "existing Rail at {:?} total {}",
            //             path_area_game_point,
            //             total_rail,
            //         );
            //         Pixel::IronOre
            //     }
            //     Pixel::IronOre => Pixel::IronOre,
            //     _ => Pixel::Rail,
            // };
            // if special_endpoint_pixels.contains(&path_area_point) {
            //     new_pixel = Pixel::CopperOre;
            // }
            // surface.set_pixel_point_u32(new_pixel, path_area_point);
        }
    }
    debug!("wrote {} rail", total_rail);
    Ok(())
}

fn get_current_unix_time_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

pub fn draw_rail(surface: &mut VSurface, rail: &Rail) {
    for point in rail.area() {
        surface.set_pixel(point, Pixel::Rail).unwrap();
    }
    surface
        .set_pixel(rail.endpoint, Pixel::Highlighter)
        .unwrap();
}

pub fn format_surface_dump(surface: &VSurface) -> String {
    let mut actual: Vec<Vec<String>> = Vec::new();
    for chunk in &surface
        .dump_pixels_xy()
        .chunks((surface.get_radius() * 2) as usize)
    {
        let actual_row = chunk
            .map(|v| match *v {
                Pixel::Empty => ".".to_string(),
                Pixel::Highlighter => "9".to_string(),
                Pixel::Rail => "5".to_string(),
                // _ => "1".to_string(),
                v => panic!("unhandled {:?}", v),
            })
            .collect();
        actual.push(actual_row);
    }
    format_dump(actual.as_slice())
}

fn format_dump(binary_surface: &[Vec<String>]) -> String {
    binary_surface
        .iter()
        .enumerate()
        .map(|(i, chunk)| {
            format!(
                "{}{}",
                chunk.iter().join(""),
                if i % 2 == 0 {
                    format!(" // {}", i)
                } else {
                    "".to_string()
                }
            )
        })
        .join("\n")
}

#[cfg(test)]
mod test {
    use crate::log_init;
    use crate::navigator::mori::{mori_start, Rail, RailDirection};
    use crate::surface::pixel::Pixel;
    use crate::surfacev::vpoint::VPoint;
    use crate::surfacev::vsurface::VSurface;
    use itertools::Itertools;

    #[test]
    fn rail_area_up_down_test() {
        const TEST_RADIUS: usize = 10;
        let mut surface = VSurface::new(TEST_RADIUS as u32);

        draw_rail(
            &mut surface,
            &Rail::new_straight(VPoint::new(-5, 5), RailDirection::Up),
        );
        draw_rail(
            &mut surface,
            &Rail::new_straight(VPoint::new(1, -5), RailDirection::Down),
        );
        // surface.set_pixel(VPoint::new(-9, -9), Pixel::Rail).unwrap();

        let actual_str = format_surface_dump(&surface);

        let expected = [
            // -10,-10       9,-10
            ".................... // 0",
            "....................",
            ".................... // 2",
            "....................",
            ".................... // 4",
            // -5
            ".11..11....91..11...",
            ".11..11....11..11... // 6",
            ".11..11....11..11...",
            ".11..11....11..11... // 8",
            ".11..11....11..11...",
            // 0
            ".11..11....11..11... // 10",
            ".11..11....11..11...",
            ".11..11....11..11... // 12",
            ".11..11....11..11...",
            ".11..11....11..11... // 14",
            // 5
            ".11..91....11..11...",
            ".11..11....11..11... // 16",
            "....................",
            ".................... // 18",
            "....................",
            // -10,9          9,9
        ]
        .join("\n");

        assert_eq!(actual_str, expected,);
    }

    #[test]
    fn rail_area_down_right_test() {
        const TEST_RADIUS: usize = 16;

        let mut surface = VSurface::new(TEST_RADIUS as u32);

        let origin_rail = Rail::new_straight(VPoint::new(0, 2), RailDirection::Down);
        draw_rail(&mut surface, &origin_rail);
        let turn_rail = origin_rail.move_right();
        draw_rail(&mut surface, &turn_rail);
        // draw_rail(&mut surface, VPoint::new(1, -5), RailDirection::Down);

        let actual_str = format_surface_dump(&surface);

        let expected = [
            // -10,-10       9,-10
            ".................... // .",
            "....................",
            ".................... // 2",
            "....................",
            ".................... // 4",
            // -5
            ".11..11....91..11...",
            ".11..11....11..11... // 6",
            ".11..11....11..11...",
            ".11..11....11..11... // 8",
            ".11..11....11..11...",
            // 0
            ".11..11....11..11... // 10",
            ".11..11....11..11...",
            ".11..11....11..11... // 12",
            ".11..11....11..11...",
            ".11..11....11..11... // 14",
            // 5
            ".11..91....11..11...",
            ".11..11....11..11... // 16",
            "....................",
            ".................... // 18",
            "....................",
            // -10,9          9,9
        ]
        .join("\n");

        assert_eq!(actual_str, expected,);
    }

    #[test]
    fn rail_area_down_left_test() {
        const TEST_RADIUS: usize = 60;
        let mut surface = VSurface::new(TEST_RADIUS as u32);

        let turn_rail = Rail::new_straight(VPoint::new(-5, 2), RailDirection::Down);
        draw_rail(&mut surface, &turn_rail);
        let turn_rail = turn_rail.move_left();
        draw_rail(&mut surface, &turn_rail);
        // let turn_rail = turn_rail.move_left();
        // draw_rail(&mut surface, &turn_rail);
        // let turn_rail = turn_rail.move_left();
        // draw_rail(&mut surface, &turn_rail);
        // let turn_rail = turn_rail.move_left();
        // draw_rail(&mut surface, &turn_rail);
        // let turn_rail = turn_rail.move_left();
        // draw_rail(&mut surface, &turn_rail);
        // draw_rail(&mut surface, VPoint::new(1, -5), RailDirection::Down);

        let actual_str = format_surface_dump(&surface);

        let expected = [
            // -10,-10       9,-10
            ".................... // .",
            "....................",
            ".................... // 2",
            "....................",
            ".................... // 4",
            // -5
            ".11..11....91..11...",
            ".11..11....11..11... // 6",
            ".11..11....11..11...",
            ".11..11....11..11... // 8",
            ".11..11....11..11...",
            // 0
            ".11..11....11..11... // 10",
            ".11..11....11..11...",
            ".11..11....11..11... // 12",
            ".11..11....11..11...",
            ".11..11....11..11... // 14",
            // 5
            ".11..91....11..11...",
            ".11..11....11..11... // 16",
            "....................",
            ".................... // 18",
            "....................",
            // -10,9          9,9
        ]
        .join("\n");

        assert_eq!(actual_str, expected,);
    }

    #[test]
    fn mori_basic_test() {
        log_init();

        const TEST_RADIUS: usize = 30;
        let mut surface = VSurface::new(TEST_RADIUS as u32);

        let start_rail = Rail::new_straight(VPoint::new(-14, 2), RailDirection::Right);
        let end_rail = start_rail
            .move_forward_step()
            .move_forward_step()
            .move_forward_step();
        // write_rail(
        //     &mut surface,
        //     &[start_rail.clone(), end_rail.clone()].to_vec(),
        // )
        // .unwrap();

        let search_area = surface.test_global_area();
        mori_start(&mut surface, start_rail, end_rail, search_area);

        let actual = format_surface_dump(&surface);
        assert_eq!("asdf", actual);
    }

    #[test]
    fn move_left_right_direction_test() {
        let origin = Rail::new_straight(VPoint::zero(), RailDirection::Up);

        assert_eq!(origin.move_left().direction, RailDirection::Left);
        assert_eq!(origin.move_right().direction, RailDirection::Right);

        let origin = Rail::new_straight(VPoint::zero(), RailDirection::Down);

        assert_eq!(origin.move_left().direction, RailDirection::Right);
        assert_eq!(origin.move_right().direction, RailDirection::Left);
    }

    //
    //     // #[test]
    //     // fn use_cloud() {
    //     //     let mut surface = Surface::new(100, 100);
    //     //     // surface.set_pixel(Pixel::IronOre, 50, 5);
    //     //     devo_start(
    //     //         &mut surface,
    //     //         Rail::new_straight(PointU32 { x: 15, y: 15 }, RailDirection::Right),
    //     //         Rail::new_straight(PointU32 { x: 85, y: 15 }, RailDirection::Right),
    //     //     );
    //     //     surface.save(Path::new("work/test"))
    //     // }
    //
    //     #[test]
    //     fn operator() {
    //         let mut surface = Surface::new(200, 200);
    //         let mut path: Vec<Rail> = Vec::new();
    //
    //         let origin = PointU32 { x: 75, y: 75 };
    //         surface.set_pixel_point_u32(Pixel::Highlighter, origin);
    //
    //         path.extend(make_dash_left(origin));
    //         path.extend(make_dash_right(origin));
    //         path.extend(make_dash_up(origin));
    //         path.extend(make_dash_down(origin));
    //         //
    //         path.extend(make_left_side_left_l(origin));
    //         path.extend(make_left_side_right_l(origin));
    //         //
    //         path.extend(make_right_side_left_l(origin));
    //         path.extend(make_right_side_right_l(origin));
    //         //
    //         path.extend(make_up_side_left_l(origin));
    //         path.extend(make_up_side_right_l(origin));
    //         //
    //         path.extend(make_down_side_left_l(origin));
    //         path.extend(make_down_side_right_l(origin));
    //         //
    //         path.extend(make_l_alone(origin));
    //         path.extend(make_r_alone(origin));
    //         write_rail(&mut surface, &path);
    //
    //         surface.save(Path::new("work/test"))
    //     }
    //
    //     const DASH_STEP_SIZE: u32 = 6;
    //
    //     fn make_dash_left(origin: PointU32) -> Vec<Rail> {
    //         let mut path = Vec::new();
    //
    //         path.push(Rail::new_straight(
    //             PointU32 {
    //                 x: origin.x - (RAIL_STEP_SIZE * DASH_STEP_SIZE),
    //                 y: origin.y,
    //             },
    //             RailDirection::Left,
    //         ));
    //
    //         path
    //     }
    //
    //     fn make_dash_right(origin: PointU32) -> Vec<Rail> {
    //         let mut path = Vec::new();
    //
    //         path.push(Rail::new_straight(
    //             PointU32 {
    //                 x: origin.x + (RAIL_STEP_SIZE * DASH_STEP_SIZE),
    //                 y: origin.y,
    //             },
    //             RailDirection::Left,
    //         ));
    //
    //         path
    //     }
    //
    //     fn make_dash_up(origin: PointU32) -> Vec<Rail> {
    //         let mut path = Vec::new();
    //
    //         path.push(Rail::new_straight(
    //             PointU32 {
    //                 x: origin.x,
    //                 y: origin.y + (RAIL_STEP_SIZE * DASH_STEP_SIZE),
    //             },
    //             RailDirection::Up,
    //         ));
    //
    //         path
    //     }
    //
    //     fn make_dash_down(origin: PointU32) -> Vec<Rail> {
    //         let mut path = Vec::new();
    //
    //         path.push(Rail::new_straight(
    //             PointU32 {
    //                 x: origin.x,
    //                 y: origin.y - (RAIL_STEP_SIZE * DASH_STEP_SIZE),
    //             },
    //             RailDirection::Down,
    //         ));
    //
    //         path
    //     }
    //
    //     fn make_left_side_left_l(origin: PointU32) -> Vec<Rail> {
    //         let mut path = Vec::new();
    //         path.push(Rail::new_straight(
    //             PointU32 {
    //                 x: origin.x - (RAIL_STEP_SIZE * DASH_STEP_SIZE),
    //                 y: origin.y - (RAIL_STEP_SIZE * 2),
    //             },
    //             RailDirection::Left,
    //         ));
    //         path.push(path.last().unwrap().move_left().unwrap());
    //         path.push(path.last().unwrap().move_forward().unwrap());
    //
    //         // for i in 0..(RAIL_STEP_SIZE * 2) {
    //         //     surface.set_pixel(Pixel::Stone, 30, 35 + i);
    //         // }
    //         path
    //     }
    //
    //     fn make_left_side_right_l(origin: PointU32) -> Vec<Rail> {
    //         let mut path = Vec::new();
    //         path.push(Rail::new_straight(
    //             PointU32 {
    //                 x: origin.x - (RAIL_STEP_SIZE * DASH_STEP_SIZE),
    //                 y: origin.y + (RAIL_STEP_SIZE * 2),
    //             },
    //             RailDirection::Left,
    //         ));
    //         // path.push(path.last().unwrap().move_forward().unwrap());
    //         path.push(path.last().unwrap().move_right().unwrap());
    //         path.push(path.last().unwrap().move_forward().unwrap());
    //         path
    //     }
    //
    //     fn make_right_side_left_l(origin: PointU32) -> Vec<Rail> {
    //         let mut path = Vec::new();
    //         path.push(Rail::new_straight(
    //             PointU32 {
    //                 x: origin.x + (RAIL_STEP_SIZE * DASH_STEP_SIZE),
    //                 y: origin.y + (RAIL_STEP_SIZE * 2),
    //             },
    //             RailDirection::Right,
    //         ));
    //
    //         path.push(path.last().unwrap().move_forward().unwrap());
    //
    //         // for i in 0..(RAIL_STEP_SIZE * 2) {
    //         //     surface.set_pixel(Pixel::Stone, 30, 35 + i);
    //         // }
    //         path
    //     }
    //
    //     fn make_right_side_right_l(origin: PointU32) -> Vec<Rail> {
    //         let mut path = Vec::new();
    //         path.push(Rail::new_straight(
    //             PointU32 {
    //                 x: origin.x + (RAIL_STEP_SIZE * DASH_STEP_SIZE),
    //                 y: origin.y - (RAIL_STEP_SIZE * 2),
    //             },
    //             RailDirection::Right,
    //         ));
    //         // path.push(path.last().unwrap().move_forward().unwrap());
    //         path.push(path.last().unwrap().move_right().unwrap());
    //         path.push(path.last().unwrap().move_forward().unwrap());
    //         path
    //     }
    //
    //     fn make_up_side_left_l(origin: PointU32) -> Vec<Rail> {
    //         let mut path = Vec::new();
    //         path.push(Rail::new_straight(
    //             PointU32 {
    //                 x: origin.x - (RAIL_STEP_SIZE * 2),
    //                 y: origin.y + (RAIL_STEP_SIZE * DASH_STEP_SIZE),
    //             },
    //             RailDirection::Up,
    //         ));
    //         path.push(path.last().unwrap().move_left().unwrap());
    //         path.push(path.last().unwrap().move_forward().unwrap());
    //
    //         // for i in 0..(RAIL_STEP_SIZE * 2) {
    //         //     surface.set_pixel(Pixel::Stone, 30, 35 + i);
    //         // }
    //         path
    //     }
    //
    //     fn make_up_side_right_l(origin: PointU32) -> Vec<Rail> {
    //         let mut path = Vec::new();
    //         path.push(Rail::new_straight(
    //             PointU32 {
    //                 x: origin.x + (RAIL_STEP_SIZE * 2),
    //                 y: origin.y + (RAIL_STEP_SIZE * DASH_STEP_SIZE),
    //             },
    //             RailDirection::Up,
    //         ));
    //         // path.push(path.last().unwrap().move_forward().unwrap());
    //         path.push(path.last().unwrap().move_right().unwrap());
    //         path.push(path.last().unwrap().move_forward().unwrap());
    //         path
    //     }
    //
    //     fn make_down_side_left_l(origin: PointU32) -> Vec<Rail> {
    //         let mut path = Vec::new();
    //         path.push(Rail::new_straight(
    //             PointU32 {
    //                 x: origin.x + (RAIL_STEP_SIZE * 2),
    //                 y: origin.y - (RAIL_STEP_SIZE * DASH_STEP_SIZE),
    //             },
    //             RailDirection::Down,
    //         ));
    //         path.push(path.last().unwrap().move_left().unwrap());
    //         path.push(path.last().unwrap().move_forward().unwrap());
    //
    //         // for i in 0..(RAIL_STEP_SIZE * 2) {
    //         //     surface.set_pixel(Pixel::Stone, 30, 35 + i);
    //         // }
    //         path
    //     }
    //
    //     fn make_down_side_right_l(origin: PointU32) -> Vec<Rail> {
    //         let mut path = Vec::new();
    //         path.push(Rail::new_straight(
    //             PointU32 {
    //                 x: origin.x - (RAIL_STEP_SIZE * 2),
    //                 y: origin.y - (RAIL_STEP_SIZE * DASH_STEP_SIZE),
    //             },
    //             RailDirection::Down,
    //         ));
    //         // path.push(path.last().unwrap().move_forward().unwrap());
    //         path.push(path.last().unwrap().move_right().unwrap());
    //         path.push(path.last().unwrap().move_forward().unwrap());
    //         path
    //     }
    //
    //     fn make_l_alone(origin: PointU32) -> Vec<Rail> {
    //         let mut path = Vec::new();
    //         path.push(Rail::new_straight(
    //             PointU32 {
    //                 x: origin.x - (RAIL_STEP_SIZE * 2),
    //                 y: origin.y + 100 - (RAIL_STEP_SIZE * DASH_STEP_SIZE),
    //             },
    //             RailDirection::Down,
    //         ));
    //         path.push(path.last().unwrap().move_left().unwrap());
    //
    //         // [path.last().unwrap().clone()].into()
    //         path
    //     }
    //
    //     fn make_r_alone(origin: PointU32) -> Vec<Rail> {
    //         let mut path = Vec::new();
    //         path.push(Rail::new_straight(
    //             PointU32 {
    //                 x: origin.x - (RAIL_STEP_SIZE * 2),
    //                 y: origin.y + 140 - (RAIL_STEP_SIZE * DASH_STEP_SIZE),
    //             },
    //             RailDirection::Down,
    //         ));
    //         path.push(path.last().unwrap().move_right().unwrap());
    //
    //         // Vec::from([path.last().unwrap().clone()])
    //         path
    //     }
    //
    //     #[test]
    //     fn surface_vs_opencv() {
    //         let mut surface = Surface::new(100, 100);
    //         let center = PointU32 { x: 15, y: 15 };
    //
    //         surface.draw_square(&Pixel::Stone, 5, &center);
    //
    //         let mut img = surface.get_buffer_to_cv();
    //         match img.at_2d_mut::<u8>(center.x as i32, center.y as i32) {
    //             Ok(e) => *e = Pixel::EdgeWall as u8,
    //             Err(e) => panic!("error {}", e),
    //         }
    //         surface.set_buffer_from_cv(img);
    //
    //         surface.save(Path::new("work/test2"))
    //     }
    //
    //     #[test]
    //     fn closed_circle_left_test() {
    //         let mut surface = Surface::new(100, 100);
    //         let start = PointU32 { x: 50, y: 50 };
    //
    //         let mut rail = Vec::from([Rail::new_straight(start, RailDirection::Right)]);
    //         rail.push(rail.last().unwrap().move_left().unwrap());
    //         rail.push(rail.last().unwrap().move_left().unwrap());
    //         rail.push(rail.last().unwrap().move_left().unwrap());
    //         rail.push(rail.last().unwrap().move_left().unwrap());
    //         assert_eq_rail(
    //             rail.first().unwrap(),
    //             rail.last().unwrap(),
    //             &mut surface,
    //             &rail,
    //             |r| r.x + r.y,
    //         );
    //     }
    //
    //     #[test]
    //     fn closed_circle_right_test() {
    //         let mut surface = Surface::new(100, 100);
    //         let start = PointU32 { x: 50, y: 50 };
    //
    //         // we are on X going up and down Y for turns
    //         let mut rail = Vec::from([Rail::new_straight(start, RailDirection::Right)]);
    //         rail.push(rail.last().unwrap().move_right().unwrap());
    //         rail.push(rail.last().unwrap().move_right().unwrap());
    //         rail.push(rail.last().unwrap().move_right().unwrap());
    //         rail.push(rail.last().unwrap().move_right().unwrap());
    //         assert_eq_rail(
    //             rail.first().unwrap(),
    //             rail.last().unwrap(),
    //             &mut surface,
    //             &rail,
    //             |r| r.y,
    //         );
    //     }
    //
    //     #[test]
    //     fn return_to_center_line_right_test() {
    //         let mut surface = Surface::new(100, 100);
    //         let start = PointU32 { x: 30, y: 50 };
    //
    //         let mut rail = Vec::from([Rail::new_straight(start, RailDirection::Right)]);
    //         rail.push(rail.last().unwrap().move_right().unwrap());
    //         rail.push(rail.last().unwrap().move_left().unwrap());
    //         rail.push(rail.last().unwrap().move_left().unwrap());
    //         rail.push(rail.last().unwrap().move_right().unwrap());
    //         assert_eq_rail(
    //             rail.first().unwrap(),
    //             rail.last().unwrap(),
    //             &mut surface,
    //             &rail,
    //             |r| r.y,
    //         );
    //     }
    //
    //     #[test]
    //     fn return_to_center_line_left_test() {
    //         let mut surface = Surface::new(100, 100);
    //         let start = PointU32 { x: 30, y: 50 };
    //
    //         let mut rail: Vec<Rail> = Vec::from([Rail::new_straight(start, RailDirection::Right)]);
    //         rail.push(rail.last().unwrap().move_left().unwrap());
    //         rail.push(rail.last().unwrap().move_right().unwrap());
    //         rail.push(rail.last().unwrap().move_right().unwrap());
    //         rail.push(rail.last().unwrap().move_left().unwrap());
    //         assert_eq_rail(
    //             rail.first().unwrap(),
    //             rail.last().unwrap(),
    //             &mut surface,
    //             &rail,
    //             |r| r.y,
    //         );
    //     }
    //
    //     #[test]
    //     fn centerline_vs_left_right_distance_test() {
    //         let mut surface = Surface::new(100, 100);
    //         let start = PointU32 { x: 30, y: 50 };
    //
    //         let mut wavy_rail: Vec<Rail> = Vec::from([Rail::new_straight(start, RailDirection::Right)]);
    //         wavy_rail.push(wavy_rail.last().unwrap().move_left().unwrap());
    //         wavy_rail.push(wavy_rail.last().unwrap().move_right().unwrap());
    //         wavy_rail.push(wavy_rail.last().unwrap().move_right().unwrap());
    //         wavy_rail.push(wavy_rail.last().unwrap().move_left().unwrap());
    //
    //         let mut straight_rail: Vec<Rail> = Vec::from([Rail::new_straight(
    //             PointU32 {
    //                 x: start.x,
    //                 y: start.y - 20,
    //             },
    //             RailDirection::Right,
    //         )]);
    //         straight_rail.push(straight_rail.last().unwrap().move_forward().unwrap());
    //         straight_rail.push(straight_rail.last().unwrap().move_forward().unwrap());
    //         straight_rail.push(straight_rail.last().unwrap().move_forward().unwrap());
    //         straight_rail.push(straight_rail.last().unwrap().move_forward().unwrap());
    //
    //         let mut all_rail = Vec::new();
    //         wavy_rail.clone().into_iter().for_each(|v| all_rail.push(v));
    //         straight_rail
    //             .clone()
    //             .into_iter()
    //             .for_each(|v| all_rail.push(v));
    //
    //         assert_eq_rail(
    //             straight_rail.last().unwrap(),
    //             wavy_rail.last().unwrap(),
    //             &mut surface,
    //             &all_rail,
    //             |r| r.x,
    //         );
    //
    //         for i in 0..wavy_rail.len() {
    //             assert_eq_rail(
    //                 &straight_rail[i],
    //                 &wavy_rail[i],
    //                 &mut surface,
    //                 &all_rail,
    //                 |r| r.x,
    //             );
    //         }
    //     }
    //
    //     fn assert_eq_rail<T>(a: &Rail, b: &Rail, surface: &mut Surface, all_rail: &Vec<Rail>, test: T)
    //     where
    //         T: Fn(&RailPoint) -> i32,
    //     {
    //         let compare_a = test(&a.endpoint);
    //         let compare_b = test(&b.endpoint);
    //         if compare_a != compare_b {
    //             write_rail(surface, all_rail);
    //             surface.save(Path::new("work/test4"));
    //
    //             assert_eq!(
    //                 compare_a, compare_b,
    //                 "point left {:?} right {:?}",
    //                 a.endpoint, b.endpoint
    //             );
    //         }
    //     }
}
