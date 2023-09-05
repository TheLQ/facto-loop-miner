use crate::navigator::devo_bias::{calculate_bias_for_point, RailAction};
use crate::navigator::resource_cloud::ResourceCloud;
use crate::state::machine::StepParams;
use crate::surface::patch::DiskPatch;
use crate::surface::pixel::Pixel;
use crate::surface::surface::{PointU32, Surface};
use crate::LOCALE;
use num_format::ToFormattedString;
use pathfinding::prelude::astar;
use rayon::prelude::*;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

pub fn devo_start(surface: &mut Surface, mut start: Rail, mut end: Rail, params: &StepParams) {
    let start_time = Instant::now();

    start = start.round();
    end = end.round();

    // end.endpoint.y = end.endpoint.y + 800;

    // // come in as far away as possible
    // // optimize area around base
    end = {
        let mut counter = 0;
        let mut new = end.clone();
        new = new.move_force_rotate_clockwise(2);
        while let Some(next) = new.move_forward().and_then(|v| v.into_buildable(surface)) {
            new = next;
            counter = counter + 0;
        }
        new = new.move_force_rotate_clockwise(2);
        // manual 1, may be unnessesary
        new = new.move_forward().unwrap();
        for _ in 0..RAIL_STEP_SIZE {
            new = new.move_forward().unwrap();
        }
        println!("moves {} from {:?} to {:?}", counter, end, new);

        // surface.draw_square(&Pixel::Stone, 100, new.endpoint.to_point_u32().unwrap());

        new
    };

    // let path = Vec::from([end]);

    let mut valid_destinations: Vec<Rail> = Vec::new();
    for width in 0..RAIL_STEP_SIZE {
        for height in 0..RAIL_STEP_SIZE {
            let mut next = end.clone();
            next.endpoint.x = &next.endpoint.x + (width as i32 * 2);
            next.endpoint.y = &next.endpoint.y + (height as i32 * 2);
            valid_destinations.push(next);
        }
    }

    let patches = DiskPatch::load_from_step_history(&params.step_history_out_dirs);
    let resource_cloud = ResourceCloud::from_patches(&patches);
    // let resource_cloud = ResourceCloud::default();

    println!("Devo start {:?} end {:?}", start, end);
    let (path, path_size) = astar(
        &start,
        |p| p.successors(surface, &end, &resource_cloud),
        |_p| 1,
        |p| valid_destinations.contains(p),
    )
    .unwrap();
    println!("built path {} long with {}", path.len(), path_size);

    write_rail(surface, path);

    let end_time = Instant::now();
    let duration = end_time - start_time;

    println!(
        "+++ Devo duration {}",
        duration.as_millis().to_formatted_string(&LOCALE)
    );

    println!(
        "metric successors called {}",
        METRIC_SUCCESSOR
            .load(Ordering::Relaxed)
            .to_formatted_string(&LOCALE)
    )
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
                // println!("source {:?}", source_rail);

                let first_leg = source_rail.move_forward()?;
                // println!("first_leg {:?}", first_leg);

                let is_left_turn =
                    source_rail.move_force_rotate_clockwise(1).direction == self.direction;
                if is_left_turn {
                    let dog_leg = first_leg.move_force_forward(DUAL_RAIL_SIZE - 1);
                    res.extend(dog_leg.area()?);
                    // println!("dog_leg {:?}", dog_leg);
                }

                let mut second_leg = first_leg.clone();
                second_leg.direction = self.direction.clone();
                second_leg = second_leg.move_forward()?;
                // println!("second_leg {:?}", second_leg);

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
        surface: &Surface,
        end: &Rail,
        resource_cloud: &ResourceCloud,
    ) -> Vec<(Self, u32)> {
        METRIC_SUCCESSOR.fetch_add(1, Ordering::Relaxed);

        let mut res = Vec::new();
        if let Some(rail) = self.move_forward().and_then(|v| v.into_buildable(surface)) {
            let cost =
                calculate_bias_for_point(RailAction::Straight, self, &rail, end, resource_cloud);
            res.push((rail, cost))
        }

        if let Some(rail) = self.move_left().and_then(|v| v.into_buildable(surface)) {
            let cost =
                calculate_bias_for_point(RailAction::TurnLeft, self, &rail, end, resource_cloud);
            res.push((rail, cost))
        }
        if let Some(rail) = self.move_right().and_then(|v| v.into_buildable(surface)) {
            let cost =
                calculate_bias_for_point(RailAction::TurnRight, self, &rail, end, resource_cloud);
            res.push((rail, cost))
        }
        // println!(
        //     "for {:?} found {}",
        //     &self,
        //     res.iter().map(|(rail, _)| format!("{:?}", rail)).join("|")
        // );
        res
    }

    fn into_buildable(self, surface: &Surface) -> Option<Self> {
        match 1 {
            1 => self.into_buildable_sequential(surface),
            2 => self.into_buildable_parallel(surface),
            _ => panic!("0"),
        }
    }

    fn into_buildable_sequential(self, surface: &Surface) -> Option<Self> {
        if let Some(area) = self.area() {
            area.iter()
                .filter_map(map_buildable_points)
                .flatten()
                .map(|game_pont| is_buildable_point_u32_take(surface, game_pont))
                .reduce(|total, is_buildable| total.and(is_buildable))
                .unwrap()
                .map(|_| self)
        } else {
            None
        }
    }

    fn into_buildable_parallel(self, surface: &Surface) -> Option<Self> {
        // observations: way slower than sequential, especially in --release mode
        if let Some(area) = self.area() {
            let area_buildable_opt: Vec<Option<PointU32>> = area
                .par_iter()
                .filter_map(map_buildable_points)
                .flat_map_iter(|v| v)
                .map(|game_pont| is_buildable_point_u32_take(surface, game_pont))
                .collect();

            area_buildable_opt
                .into_iter()
                .reduce(|total, is_buildable| total.and(is_buildable))
                .unwrap()
                .map(|_| self)
        } else {
            None
        }
    }
}

fn map_buildable_points<'r>(rail: &RailPoint) -> Option<Vec<PointU32>> {
    let game_points: Vec<PointU32> = rail.to_game_points();
    if game_points.len() == 4 {
        Some(game_points)
    } else {
        None
    }
}

impl RailPoint {
    fn round(&self) -> Self {
        let mut next = self.clone();
        next.x = if next.x % 2 == 0 { next.x + 1 } else { next.x };
        next.y = if next.y % 2 == 0 { next.y + 1 } else { next.y };
        next
    }

    fn to_game_points(&self) -> Vec<PointU32> {
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
        .map(|v| v.to_point_u32())
        .filter_map(|v| v)
        .collect()
    }

    fn is_negative(&self) -> bool {
        self.x < 0 || self.y < 0
    }

    fn to_point_u32(&self) -> Option<PointU32> {
        if self.is_negative() {
            None
        } else {
            Some(PointU32 {
                x: self.x.try_into().unwrap(),
                y: self.y.try_into().unwrap(),
            })
        }
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
        Pixel::Empty | Pixel::EdgeWall => Some(point),
        _existing => {
            // println!("blocked at {:?} by {:?}", &position, existing);
            None
        }
    }
}

fn is_buildable_point_u32<'p>(surface: &Surface, point: &'p PointU32) -> Option<&'p PointU32> {
    if !surface.xy_in_range_point_u32(point) {
        return None;
    }
    match surface.get_pixel_point_u32(point) {
        Pixel::Empty | Pixel::EdgeWall => Some(point),
        _existing => {
            // println!("blocked at {:?} by {:?}", &position, existing);
            None
        }
    }
}

pub fn write_rail(surface: &mut Surface, path: Vec<Rail>) {
    let special_endpoint_pixels: Vec<PointU32> = path
        .iter()
        .map(|v| v.endpoint.to_point_u32().unwrap())
        .collect();

    let mut total_rail = 0;
    for path_rail in path {
        if let Some(path_area) = path_rail.area() {
            for path_area_rail_point in path_area {
                total_rail = total_rail + 1;
                for path_area_game_point in path_area_rail_point.to_game_points() {
                    let mut new_pixel = match surface.get_pixel_point_u32(&path_area_game_point) {
                        Pixel::Rail => {
                            println!(
                                "existing Rail at {:?} total {}",
                                path_area_game_point, total_rail
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
    use crate::navigator::devo::{write_rail, Rail, RailDirection, RAIL_STEP_SIZE};
    use crate::surface::pixel::Pixel;
    use crate::surface::surface::{PointU32, Surface};
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

        write_rail(&mut surface, path);

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
        path.push(path.last().unwrap().move_left().unwrap());
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

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
