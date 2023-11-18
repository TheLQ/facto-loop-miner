use crate::navigator::mori_cost::{calculate_cost_for_point, RailAction};
use crate::navigator::resource_cloud::ResourceCloud;
use crate::simd_diff::SurfaceDiff;
use crate::state::machine::StepParams;
use crate::surface::patch::DiskPatch;
use crate::surface::pixel::Pixel;
use crate::surface::surface::{PointU32, Surface};
use crate::LOCALE;
use num_format::ToFormattedString;
use opencv::prelude::*;
use pathfinding::prelude::astar_mori;
use rayon::prelude::*;
use std::arch::x86_64::__m256i;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

/// Pathfinder v1, Mori Calliope
///
/// Makes a dual rail + spacing, +6 straight or 90 degree turning, path of rail from start to end.
/// Without collisions into any point on the Surface.
pub fn mori_start(
    surface: &Surface,
    mut start: Rail,
    mut end: Rail,
    step_params: &StepParams,
) -> Option<Vec<Rail>> {
    let start_time = Instant::now();

    start = start.round();
    end = end.round();

    let mut valid_destinations: Vec<Rail> = Vec::new();
    for width in 0..RAIL_STEP_SIZE {
        for height in 0..RAIL_STEP_SIZE {
            let mut next = end.clone();
            next.endpoint.x = &next.endpoint.x + (width as i32 * 2);
            next.endpoint.y = &next.endpoint.y + (height as i32 * 2);
            valid_destinations.push(next);
        }
    }

    let patches = DiskPatch::load_from_step_history(&step_params.step_history_out_dirs);
    let resource_cloud = ResourceCloud::from_patches(&patches);

    let mut working_buffer = surface.surface_diff();

    tracing::debug!("Mori start {:?} end {:?}", start, end);
    // Forked function only passes on the parents, used for limits
    let pathfind = astar_mori(
        &start,
        |(p, parents, total_cost)| {
            p.successors(
                parents,
                total_cost,
                surface,
                &end,
                &resource_cloud,
                &mut working_buffer,
            )
        },
        |_p| 1,
        |p| valid_destinations.contains(p),
    );
    let mut result = None;
    if let Some(pathfind) = pathfind {
        let (path, path_size) = pathfind;
        tracing::debug!("built path {} long with {}", path.len(), path_size);

        result = Some(path);
    } else {
        tracing::debug!("failed to pathfind from {:?} to {:?}", start, end);
    }

    let end_time = Instant::now();
    let duration = end_time - start_time;
    tracing::debug!(
        "+++ Mori duration {}",
        duration.as_millis().to_formatted_string(&LOCALE),
    );

    tracing::debug!(
        "metric successors called {}",
        METRIC_SUCCESSOR
            .swap(0, Ordering::Relaxed)
            .to_formatted_string(&LOCALE),
    );

    result
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RailPoint {
    pub(crate) x: i32,
    pub(crate) y: i32,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RailMode {
    Straight,
    Curved(RailPoint, RailDirection),
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RailDirection {
    Up,
    Down,
    Left,
    Right,
}

const RAIL_DIRECTION_CLOCKWISE: [RailDirection; 4] = [
    RailDirection::Up,
    RailDirection::Right,
    RailDirection::Down,
    RailDirection::Left,
];

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Rail {
    pub endpoint: RailPoint,
    pub direction: RailDirection,
    pub mode: RailMode,
}

const DUAL_RAIL_SIZE: u32 = 3;

static METRIC_SUCCESSOR: AtomicU64 = AtomicU64::new(0);

const RAIL_STEP_SIZE: u32 = 6;

impl Rail {
    pub fn new_straight(point: PointU32, direction: RailDirection) -> Self {
        let res = Rail {
            endpoint: RailPoint::from_point_u32(point).round(),
            mode: RailMode::Straight,
            direction,
        };

        res
    }

    fn area(&self) -> Option<Vec<RailPoint>> {
        let mut res = Vec::new();
        match &self.mode {
            RailMode::Straight => {
                for step_forward in 0..RAIL_STEP_SIZE {
                    let mut next = self.clone();
                    // must cover 0..1 -like area
                    next = next.move_force_rotate_clockwise(2);
                    next = next.move_force_forward(step_forward);
                    for adjacent in 0..DUAL_RAIL_SIZE {
                        let mut next = next.move_force_rotate_clockwise(1);
                        next = next.move_force_forward(adjacent);

                        if next.endpoint.is_negative() {
                            return None;
                        }

                        res.push(next.endpoint);
                    }
                }
            }
            RailMode::Curved(source_point, source_direction) => {
                let source_rail = Rail {
                    endpoint: source_point.clone(),
                    direction: source_direction.clone(),
                    mode: RailMode::Straight,
                };
                // tracing::debug!("source {:?}", source_rail);

                let first_leg = source_rail.move_forward()?;
                // tracing::debug!("first_leg {:?}", first_leg);

                let is_left_turn =
                    source_rail.move_force_rotate_clockwise(1).direction == self.direction;
                if is_left_turn {
                    let dog_leg = first_leg.move_force_forward(DUAL_RAIL_SIZE - 1);
                    res.extend(dog_leg.area()?);
                    // tracing::debug!("dog_leg {:?}", dog_leg);
                }

                let mut second_leg = first_leg.clone();
                second_leg.direction = self.direction.clone();
                second_leg = second_leg.move_forward()?;
                // tracing::debug!("second_leg {:?}", second_leg);

                res.extend(first_leg.area()?);

                res.extend(second_leg.area()?);

                // very first row is the source's
                for width in 1..RAIL_STEP_SIZE {
                    for height in 0..RAIL_STEP_SIZE {
                        let mut next = source_rail.clone();
                        next.move_force_forward_mut(width);
                        next.direction = self.direction.clone();
                        next.move_force_forward_mut(height);

                        if next.endpoint.is_negative() {
                            return None;
                        }

                        res.push(next.endpoint);
                    }
                }

                res.sort();
                res.dedup();
            }
        }

        // let before = next.area.len();
        // next.area.dedup();
        // let after = next.area.len();
        // if before != after {
        //     panic!("broken area check {} \n\n {}", before, after);
        // }

        Some(res)
    }

    fn round(&self) -> Self {
        let mut next = self.clone();
        next.endpoint = next.endpoint.round();
        next
    }

    #[allow(dead_code)]
    pub(crate) fn distance(&self, other: &Rail) -> u32 {
        let a = &self.endpoint;
        let b = &other.endpoint;
        a.x.abs_diff(b.x) + a.y.abs_diff(b.y)
    }

    fn move_force_rotate_clockwise(&self, rotations: usize) -> Self {
        let mut directions = RAIL_DIRECTION_CLOCKWISE.iter().cycle();

        while directions.next().unwrap() != &self.direction {}

        let mut new_direction = directions.next().unwrap();
        for _ in 1..rotations {
            new_direction = directions.next().unwrap();
        }

        let mut next = self.clone();
        next.direction = new_direction.clone();
        next
    }

    pub fn move_backwards_toward_water(
        &self,
        surface: &Surface,
        working_buffer: &mut SurfaceDiff,
    ) -> Self {
        // // come in as far away as possible
        // // optimize area around base
        let mut counter = 0;
        let mut next = self.clone();
        next = next.move_force_rotate_clockwise(2);
        // TODO
        loop {
            let next_rail = next.move_force_forward(1);
            if let Some(next_rail) = next_rail.into_buildable(surface, working_buffer) {
                next = next_rail;
            } else {
                break;
            }
        }
        // point back in original direction
        next = next.move_force_rotate_clockwise(2);
        // back away from water with a full step
        next = next.move_forward().unwrap();
        // full step again for spacing (eg in a cove)
        next = next.move_forward().unwrap();

        tracing::debug!("move backwards {} from {:?} to {:?}", counter, self, next);

        next
    }

    pub fn move_forward(&self) -> Option<Self> {
        let mut next = self.clone();
        next.mode = RailMode::Straight;
        next.move_force_forward_mut(RAIL_STEP_SIZE);
        if next.endpoint.is_negative() {
            None
        } else {
            Some(next)
        }
    }

    fn move_force_forward(&self, steps: u32) -> Rail {
        let mut cur = self.clone();
        cur.move_force_forward_mut(steps);
        cur
    }

    fn move_force_forward_mut(&mut self, steps: u32) {
        // rail is 2x2
        let steps = steps * 2;
        match self.direction {
            RailDirection::Up => self.endpoint.y = self.endpoint.y + steps as i32,
            RailDirection::Down => self.endpoint.y = self.endpoint.y - steps as i32,
            RailDirection::Left => self.endpoint.x = self.endpoint.x - steps as i32,
            RailDirection::Right => self.endpoint.x = self.endpoint.x + steps as i32,
        };
    }

    /// may be negative
    fn distance_in_direction_to_point(&self, end: &PointU32) -> i32 {
        match self.direction {
            RailDirection::Up => end.y as i32 - self.endpoint.y,
            RailDirection::Down => -(end.y as i32 - self.endpoint.y),
            RailDirection::Left => end.x as i32 - self.endpoint.x,
            RailDirection::Right => -(end.x as i32 - self.endpoint.x),
        }
    }

    fn move_left(&self) -> Option<Self> {
        self.move_rotating(3)
    }

    fn move_right(&self) -> Option<Self> {
        self.move_rotating(1)
    }

    fn move_rotating(&self, rotation_steps: usize) -> Option<Self> {
        let mut next = self.clone();
        next.move_force_forward_mut(RAIL_STEP_SIZE);
        next = next.move_force_rotate_clockwise(rotation_steps);
        next.move_force_forward_mut(RAIL_STEP_SIZE);

        if next.endpoint.is_negative() {
            None
        } else {
            next.mode = RailMode::Curved(self.endpoint.clone(), self.direction.clone());
            Some(next)
        }
    }

    fn successors(
        &self,
        parents: Vec<Rail>,
        total_cost: u32,
        surface: &Surface,
        end: &Rail,
        resource_cloud: &ResourceCloud,
        working_buffer: &mut SurfaceDiff,
    ) -> Vec<(Self, u32)> {
        // if parents.len() > 100 {
        //     return Vec::new();
        // }
        // tracing::debug!("testing {:?}", self);

        {
            let cur = METRIC_SUCCESSOR.fetch_add(1, Ordering::Relaxed);
            if cur % 100_000 == 0 {
                tracing::debug!(
                    "successor {} spot parents {} size {}x{}",
                    cur.to_formatted_string(&LOCALE),
                    parents.len(),
                    surface.width,
                    surface.height,
                );
            }
        }

        let mut res = Vec::new();
        if let Some(rail) = self
            .move_forward()
            .and_then(|v| v.into_buildable(surface, working_buffer))
        {
            let cost =
                calculate_cost_for_point(RailAction::Straight, self, &rail, end, resource_cloud);
            if !(rail.distance(end) < 400 && rail.direction != end.direction) {
                res.push((rail, cost))
            }
        }

        if let Some(rail) = self
            .move_left()
            .and_then(|v| v.into_buildable(surface, working_buffer))
        {
            let cost =
                calculate_cost_for_point(RailAction::TurnLeft, self, &rail, end, resource_cloud);
            if !(rail.distance(end) < 400 && rail.direction != end.direction) {
                res.push((rail, cost))
            }
        }
        if let Some(rail) = self
            .move_right()
            .and_then(|v| v.into_buildable(surface, working_buffer))
        {
            let cost =
                calculate_cost_for_point(RailAction::TurnRight, self, &rail, end, resource_cloud);
            if !(rail.distance(end) < 400 && rail.direction != end.direction) {
                res.push((rail, cost))
            }
        }
        // tracing::debug!(
        //     "for {:?} found {}",
        //     &self,
        //     res.iter().map(|(rail, _)| format!("{:?}", rail)).join("|")
        // );
        res
    }

    fn into_buildable(self, surface: &Surface, working_buffer: &mut SurfaceDiff) -> Option<Self> {
        const SIZE: u32 = 0;
        if self.endpoint.x < 4000 {
            None
        } else {
            match 1 {
                1 => self.into_buildable_sequential(surface),
                // 2 => self.into_buildable_parallel(surface),
                3 => self.into_buildable_avx(surface, working_buffer),
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
        }
    }

    fn into_buildable_sequential(self, surface: &Surface) -> Option<Self> {
        if let Some(area) = self.area() {
            area.into_iter()
                .map(|v| filter_available_points(&v, surface))
                .flatten()
                .try_fold(Some(self), |acc, cur| {
                    acc.and_then(|acc| {
                        cur.and_then(|v| is_buildable_point_u32_take(surface, v).map(|_| Some(acc)))
                    })
                })
                .flatten()
            // let points: Vec<Option<PointU32>> = area.into_iter().map(|v| filter_available_points(&v, surface)).collect();
            // if points.contains(None) {
            //     return None;
            // }
            // .reduce(|total, is_buildable| total.and(is_buildable))
            // .unwrap()
            // // .flatten()
            // .map(|_| self)
        } else {
            None
        }
    }

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

    fn into_buildable_avx(
        self,
        surface: &Surface,
        working_buffer: &mut SurfaceDiff,
    ) -> Option<Self> {
        if let Some(area) = self.area() {
            if let Some(points) = area
                .into_iter()
                .map(|v| filter_available_points(&v, surface))
                .flatten()
                .map(|v| v.and_then(|v| Some(surface.xy_to_index_point_u32(v))))
                .try_collect()
            {
                if working_buffer.is_positions_free(points) {
                    return Some(self);
                }
            }
        }
        None
    }
}

fn filter_available_points<'r>(rail: &RailPoint, surface: &Surface) -> [Option<PointU32>; 4] {
    // let game_points: Vec<PointU32> = rail.to_game_points(surface);
    // if game_points.len() == 4 {
    //     Some(game_points)
    // } else {
    //     None
    // }

    let v1 = rail;

    let mut v2 = rail.clone();
    v2.x = v2.x - 1;

    let mut v3 = rail.clone();
    v3.y = v3.y - 1;

    let mut v4 = rail.clone();
    v4.x = v4.x - 1;
    v4.y = v4.y - 1;

    [
        v1.to_point_u32_surface(surface),
        v2.to_point_u32_surface(surface),
        v3.to_point_u32_surface(surface),
        v4.to_point_u32_surface(surface),
    ]

    // match (
    //     v1.to_point_u32_surface(surface),
    //     v2.to_point_u32_surface(surface),
    //     v3.to_point_u32_surface(surface),
    //     v4.to_point_u32_surface(surface),
    // ) {
    //     (Some(v1), Some(v2), Some(v3), Some(v4)) => {
    //         let mut game_points: Vec<PointU32> = Vec::new();
    //         game_points.push(v1);
    //         game_points.push(v2);
    //         game_points.push(v3);
    //         game_points.push(v4);
    //         Some(game_points)
    //     }
    //     _ => None,
    // }
}

impl RailPoint {
    fn round(&self) -> Self {
        let mut next = self.clone();
        next.x = if next.x % 2 == 0 { next.x + 1 } else { next.x };
        next.y = if next.y % 2 == 0 { next.y + 1 } else { next.y };
        next
    }

    fn to_game_points(&self, surface: &Surface) -> Vec<PointU32> {
        [
            self.clone(),
            {
                let mut v = self.clone();
                v.x = v.x - 1;
                v
            },
            {
                let mut v = self.clone();
                v.y = v.y - 1;
                v
            },
            {
                let mut v = self.clone();
                v.x = v.x - 1;
                v.y = v.y - 1;
                v
            },
        ]
        .iter()
        .filter_map(|v| v.to_point_u32_surface(surface))
        .collect()
    }

    fn is_negative(&self) -> bool {
        self.x < 0 || self.y < 0
    }

    pub fn to_point_u32(&self) -> Option<PointU32> {
        if self.is_negative() {
            None
        } else {
            Some(PointU32 {
                x: self.x.try_into().unwrap(),
                y: self.y.try_into().unwrap(),
            })
        }
    }

    fn to_point_u32_surface(&self, surface: &Surface) -> Option<PointU32> {
        self.to_point_u32().and_then(|p| {
            if p.x < surface.width && p.y < surface.height {
                Some(p)
            } else {
                None
            }
        })
    }

    pub fn from_point_u32(point: PointU32) -> Self {
        RailPoint {
            x: point.x as i32,
            y: point.y as i32,
        }
    }
}

fn is_buildable_point_u32_take(surface: &Surface, point: PointU32) -> Option<PointU32> {
    if !surface.xy_in_range_point_u32(&point) {
        return None;
    }
    match surface.get_pixel_point_u32(&point) {
        Pixel::Empty => Some(point),
        _existing => {
            // tracing::debug!("blocked at {:?} by {:?}", &position, existing);
            None
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
            // tracing::debug!("blocked at {:?} by {:?}", &position, existing);
            None
        }
    }
}

pub fn write_rail(surface: &mut Surface, path: &Vec<Rail>) {
    let special_endpoint_pixels: Vec<PointU32> = path
        .iter()
        .map(|v| v.endpoint.to_point_u32().unwrap())
        .collect();

    let mut total_rail = 0;
    for path_rail in path {
        if let Some(path_area) = path_rail.area() {
            for path_area_rail_point in path_area {
                total_rail = total_rail + 1;
                for path_area_game_point in path_area_rail_point.to_game_points(surface) {
                    let mut new_pixel = match surface.get_pixel_point_u32(&path_area_game_point) {
                        Pixel::Rail => {
                            tracing::debug!(
                                "existing Rail at {:?} total {}",
                                path_area_game_point,
                                total_rail,
                            );
                            Pixel::IronOre
                        }
                        Pixel::IronOre => Pixel::IronOre,
                        _ => Pixel::Rail,
                    };
                    if special_endpoint_pixels.contains(&path_area_game_point) {
                        new_pixel = Pixel::CopperOre;
                    }
                    surface.set_pixel_point_u32(new_pixel, path_area_game_point);
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::navigator::mori::{write_rail, Rail, RailDirection, RailPoint, RAIL_STEP_SIZE};
    use crate::surface::pixel::Pixel;
    use crate::surface::surface::{PointU32, Surface};
    use opencv::prelude::*;
    use std::path::Path;

    // #[test]
    // fn use_cloud() {
    //     let mut surface = Surface::new(100, 100);
    //     // surface.set_pixel(Pixel::IronOre, 50, 5);
    //     devo_start(
    //         &mut surface,
    //         Rail::new_straight(PointU32 { x: 15, y: 15 }, RailDirection::Right),
    //         Rail::new_straight(PointU32 { x: 85, y: 15 }, RailDirection::Right),
    //     );
    //     surface.save(Path::new("work/test"))
    // }

    #[test]
    fn operator() {
        let mut surface = Surface::new(200, 200);
        let mut path: Vec<Rail> = Vec::new();

        let origin = PointU32 { x: 75, y: 75 };
        surface.set_pixel_point_u32(Pixel::Highlighter, origin);

        path.extend(make_dash_left(origin));
        path.extend(make_dash_right(origin));
        path.extend(make_dash_up(origin));
        path.extend(make_dash_down(origin));
        //
        path.extend(make_left_side_left_l(origin));
        path.extend(make_left_side_right_l(origin));
        //
        path.extend(make_right_side_left_l(origin));
        path.extend(make_right_side_right_l(origin));
        //
        path.extend(make_up_side_left_l(origin));
        path.extend(make_up_side_right_l(origin));
        //
        path.extend(make_down_side_left_l(origin));
        path.extend(make_down_side_right_l(origin));
        //
        path.extend(make_l_alone(origin));
        path.extend(make_r_alone(origin));
        write_rail(&mut surface, &path);

        surface.save(Path::new("work/test"))
    }

    const DASH_STEP_SIZE: u32 = 6;

    fn make_dash_left(origin: PointU32) -> Vec<Rail> {
        let mut path = Vec::new();

        path.push(Rail::new_straight(
            PointU32 {
                x: origin.x - (RAIL_STEP_SIZE * DASH_STEP_SIZE),
                y: origin.y,
            },
            RailDirection::Left,
        ));

        path
    }

    fn make_dash_right(origin: PointU32) -> Vec<Rail> {
        let mut path = Vec::new();

        path.push(Rail::new_straight(
            PointU32 {
                x: origin.x + (RAIL_STEP_SIZE * DASH_STEP_SIZE),
                y: origin.y,
            },
            RailDirection::Left,
        ));

        path
    }

    fn make_dash_up(origin: PointU32) -> Vec<Rail> {
        let mut path = Vec::new();

        path.push(Rail::new_straight(
            PointU32 {
                x: origin.x,
                y: origin.y + (RAIL_STEP_SIZE * DASH_STEP_SIZE),
            },
            RailDirection::Up,
        ));

        path
    }

    fn make_dash_down(origin: PointU32) -> Vec<Rail> {
        let mut path = Vec::new();

        path.push(Rail::new_straight(
            PointU32 {
                x: origin.x,
                y: origin.y - (RAIL_STEP_SIZE * DASH_STEP_SIZE),
            },
            RailDirection::Down,
        ));

        path
    }

    fn make_left_side_left_l(origin: PointU32) -> Vec<Rail> {
        let mut path = Vec::new();
        path.push(Rail::new_straight(
            PointU32 {
                x: origin.x - (RAIL_STEP_SIZE * DASH_STEP_SIZE),
                y: origin.y - (RAIL_STEP_SIZE * 2),
            },
            RailDirection::Left,
        ));
        path.push(path.last().unwrap().move_left().unwrap());
        path.push(path.last().unwrap().move_forward().unwrap());

        // for i in 0..(RAIL_STEP_SIZE * 2) {
        //     surface.set_pixel(Pixel::Stone, 30, 35 + i);
        // }
        path
    }

    fn make_left_side_right_l(origin: PointU32) -> Vec<Rail> {
        let mut path = Vec::new();
        path.push(Rail::new_straight(
            PointU32 {
                x: origin.x - (RAIL_STEP_SIZE * DASH_STEP_SIZE),
                y: origin.y + (RAIL_STEP_SIZE * 2),
            },
            RailDirection::Left,
        ));
        // path.push(path.last().unwrap().move_forward().unwrap());
        path.push(path.last().unwrap().move_right().unwrap());
        path.push(path.last().unwrap().move_forward().unwrap());
        path
    }

    fn make_right_side_left_l(origin: PointU32) -> Vec<Rail> {
        let mut path = Vec::new();
        path.push(Rail::new_straight(
            PointU32 {
                x: origin.x + (RAIL_STEP_SIZE * DASH_STEP_SIZE),
                y: origin.y + (RAIL_STEP_SIZE * 2),
            },
            RailDirection::Right,
        ));

        path.push(path.last().unwrap().move_forward().unwrap());

        // for i in 0..(RAIL_STEP_SIZE * 2) {
        //     surface.set_pixel(Pixel::Stone, 30, 35 + i);
        // }
        path
    }

    fn make_right_side_right_l(origin: PointU32) -> Vec<Rail> {
        let mut path = Vec::new();
        path.push(Rail::new_straight(
            PointU32 {
                x: origin.x + (RAIL_STEP_SIZE * DASH_STEP_SIZE),
                y: origin.y - (RAIL_STEP_SIZE * 2),
            },
            RailDirection::Right,
        ));
        // path.push(path.last().unwrap().move_forward().unwrap());
        path.push(path.last().unwrap().move_right().unwrap());
        path.push(path.last().unwrap().move_forward().unwrap());
        path
    }

    fn make_up_side_left_l(origin: PointU32) -> Vec<Rail> {
        let mut path = Vec::new();
        path.push(Rail::new_straight(
            PointU32 {
                x: origin.x - (RAIL_STEP_SIZE * 2),
                y: origin.y + (RAIL_STEP_SIZE * DASH_STEP_SIZE),
            },
            RailDirection::Up,
        ));
        path.push(path.last().unwrap().move_left().unwrap());
        path.push(path.last().unwrap().move_forward().unwrap());

        // for i in 0..(RAIL_STEP_SIZE * 2) {
        //     surface.set_pixel(Pixel::Stone, 30, 35 + i);
        // }
        path
    }

    fn make_up_side_right_l(origin: PointU32) -> Vec<Rail> {
        let mut path = Vec::new();
        path.push(Rail::new_straight(
            PointU32 {
                x: origin.x + (RAIL_STEP_SIZE * 2),
                y: origin.y + (RAIL_STEP_SIZE * DASH_STEP_SIZE),
            },
            RailDirection::Up,
        ));
        // path.push(path.last().unwrap().move_forward().unwrap());
        path.push(path.last().unwrap().move_right().unwrap());
        path.push(path.last().unwrap().move_forward().unwrap());
        path
    }

    fn make_down_side_left_l(origin: PointU32) -> Vec<Rail> {
        let mut path = Vec::new();
        path.push(Rail::new_straight(
            PointU32 {
                x: origin.x + (RAIL_STEP_SIZE * 2),
                y: origin.y - (RAIL_STEP_SIZE * DASH_STEP_SIZE),
            },
            RailDirection::Down,
        ));
        path.push(path.last().unwrap().move_left().unwrap());
        path.push(path.last().unwrap().move_forward().unwrap());

        // for i in 0..(RAIL_STEP_SIZE * 2) {
        //     surface.set_pixel(Pixel::Stone, 30, 35 + i);
        // }
        path
    }

    fn make_down_side_right_l(origin: PointU32) -> Vec<Rail> {
        let mut path = Vec::new();
        path.push(Rail::new_straight(
            PointU32 {
                x: origin.x - (RAIL_STEP_SIZE * 2),
                y: origin.y - (RAIL_STEP_SIZE * DASH_STEP_SIZE),
            },
            RailDirection::Down,
        ));
        // path.push(path.last().unwrap().move_forward().unwrap());
        path.push(path.last().unwrap().move_right().unwrap());
        path.push(path.last().unwrap().move_forward().unwrap());
        path
    }

    fn make_l_alone(origin: PointU32) -> Vec<Rail> {
        let mut path = Vec::new();
        path.push(Rail::new_straight(
            PointU32 {
                x: origin.x - (RAIL_STEP_SIZE * 2),
                y: origin.y + 100 - (RAIL_STEP_SIZE * DASH_STEP_SIZE),
            },
            RailDirection::Down,
        ));
        path.push(path.last().unwrap().move_left().unwrap());

        // [path.last().unwrap().clone()].into()
        path
    }

    fn make_r_alone(origin: PointU32) -> Vec<Rail> {
        let mut path = Vec::new();
        path.push(Rail::new_straight(
            PointU32 {
                x: origin.x - (RAIL_STEP_SIZE * 2),
                y: origin.y + 140 - (RAIL_STEP_SIZE * DASH_STEP_SIZE),
            },
            RailDirection::Down,
        ));
        path.push(path.last().unwrap().move_right().unwrap());

        // Vec::from([path.last().unwrap().clone()])
        path
    }

    #[test]
    fn surface_vs_opencv() {
        let mut surface = Surface::new(100, 100);
        let center = PointU32 { x: 15, y: 15 };

        surface.draw_square(&Pixel::Stone, 5, &center);

        let mut img = surface.get_buffer_to_cv();
        match img.at_2d_mut::<u8>(center.x as i32, center.y as i32) {
            Ok(e) => *e = Pixel::EdgeWall as u8,
            Err(e) => panic!("error {}", e),
        }
        surface.set_buffer_from_cv(img);

        surface.save(Path::new("work/test2"))
    }

    #[test]
    fn closed_circle_left_test() {
        let mut surface = Surface::new(100, 100);
        let start = PointU32 { x: 50, y: 50 };

        let mut rail = Vec::from([Rail::new_straight(start, RailDirection::Right)]);
        rail.push(rail.last().unwrap().move_left().unwrap());
        rail.push(rail.last().unwrap().move_left().unwrap());
        rail.push(rail.last().unwrap().move_left().unwrap());
        rail.push(rail.last().unwrap().move_left().unwrap());
        assert_eq_rail(
            rail.first().unwrap(),
            rail.last().unwrap(),
            &mut surface,
            &rail,
            |r| r.x + r.y,
        );
    }

    #[test]
    fn closed_circle_right_test() {
        let mut surface = Surface::new(100, 100);
        let start = PointU32 { x: 50, y: 50 };

        // we are on X going up and down Y for turns
        let mut rail = Vec::from([Rail::new_straight(start, RailDirection::Right)]);
        rail.push(rail.last().unwrap().move_right().unwrap());
        rail.push(rail.last().unwrap().move_right().unwrap());
        rail.push(rail.last().unwrap().move_right().unwrap());
        rail.push(rail.last().unwrap().move_right().unwrap());
        assert_eq_rail(
            rail.first().unwrap(),
            rail.last().unwrap(),
            &mut surface,
            &rail,
            |r| r.y,
        );
    }

    #[test]
    fn return_to_center_line_right_test() {
        let mut surface = Surface::new(100, 100);
        let start = PointU32 { x: 30, y: 50 };

        let mut rail = Vec::from([Rail::new_straight(start, RailDirection::Right)]);
        rail.push(rail.last().unwrap().move_right().unwrap());
        rail.push(rail.last().unwrap().move_left().unwrap());
        rail.push(rail.last().unwrap().move_left().unwrap());
        rail.push(rail.last().unwrap().move_right().unwrap());
        assert_eq_rail(
            rail.first().unwrap(),
            rail.last().unwrap(),
            &mut surface,
            &rail,
            |r| r.y,
        );
    }

    #[test]
    fn return_to_center_line_left_test() {
        let mut surface = Surface::new(100, 100);
        let start = PointU32 { x: 30, y: 50 };

        let mut rail: Vec<Rail> = Vec::from([Rail::new_straight(start, RailDirection::Right)]);
        rail.push(rail.last().unwrap().move_left().unwrap());
        rail.push(rail.last().unwrap().move_right().unwrap());
        rail.push(rail.last().unwrap().move_right().unwrap());
        rail.push(rail.last().unwrap().move_left().unwrap());
        assert_eq_rail(
            rail.first().unwrap(),
            rail.last().unwrap(),
            &mut surface,
            &rail,
            |r| r.y,
        );
    }

    #[test]
    fn centerline_vs_left_right_distance_test() {
        let mut surface = Surface::new(100, 100);
        let start = PointU32 { x: 30, y: 50 };

        let mut wavy_rail: Vec<Rail> = Vec::from([Rail::new_straight(start, RailDirection::Right)]);
        wavy_rail.push(wavy_rail.last().unwrap().move_left().unwrap());
        wavy_rail.push(wavy_rail.last().unwrap().move_right().unwrap());
        wavy_rail.push(wavy_rail.last().unwrap().move_right().unwrap());
        wavy_rail.push(wavy_rail.last().unwrap().move_left().unwrap());

        let mut straight_rail: Vec<Rail> = Vec::from([Rail::new_straight(
            PointU32 {
                x: start.x,
                y: start.y - 20,
            },
            RailDirection::Right,
        )]);
        straight_rail.push(straight_rail.last().unwrap().move_forward().unwrap());
        straight_rail.push(straight_rail.last().unwrap().move_forward().unwrap());
        straight_rail.push(straight_rail.last().unwrap().move_forward().unwrap());
        straight_rail.push(straight_rail.last().unwrap().move_forward().unwrap());

        let mut all_rail = Vec::new();
        wavy_rail.clone().into_iter().for_each(|v| all_rail.push(v));
        straight_rail
            .clone()
            .into_iter()
            .for_each(|v| all_rail.push(v));

        assert_eq_rail(
            straight_rail.last().unwrap(),
            wavy_rail.last().unwrap(),
            &mut surface,
            &all_rail,
            |r| r.x,
        );

        for i in 0..wavy_rail.len() {
            assert_eq_rail(
                &straight_rail[i],
                &wavy_rail[i],
                &mut surface,
                &all_rail,
                |r| r.x,
            );
        }
    }

    fn assert_eq_rail<T>(a: &Rail, b: &Rail, surface: &mut Surface, all_rail: &Vec<Rail>, test: T)
    where
        T: Fn(&RailPoint) -> i32,
    {
        let compare_a = test(&a.endpoint);
        let compare_b = test(&b.endpoint);
        if compare_a != compare_b {
            write_rail(surface, all_rail);
            surface.save(Path::new("work/test4"));

            assert_eq!(
                compare_a, compare_b,
                "point left {:?} right {:?}",
                a.endpoint, b.endpoint
            );
        }
    }
}
