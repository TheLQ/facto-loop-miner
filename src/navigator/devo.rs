use crate::surface::pixel::Pixel;
use crate::surface::surface::{PointU32, Surface};
use crate::LOCALE;
use num_format::ToFormattedString;
use pathfinding::prelude::astar;
use std::cell::Cell;

pub fn devo_start(surface: &mut Surface, mut start: Rail, mut end: Rail) {
    start = start.round();
    end = end.round();
    println!("Devo start {:?} end {:?}", start, end);
    let (path, path_size) = astar(
        &start,
        |p| p.successors(surface, &end),
        |_p| 1,
        |p| *p == end,
    )
    .unwrap();
    println!("built path {} long with {}", path.len(), path_size);
    let mut total_rail = 0;
    for path_rail in path {
        for path_rail_point in path_rail.area {
            for path_rail_game_point in path_rail_point.to_game_points() {
                total_rail = total_rail + 1;
                if surface.get_pixel_point_u32(path_rail_game_point) == &Pixel::Rail {
                    println!("existing {:?} at {}", path_rail_point, total_rail)
                }
                surface.set_pixel_point_u32(Pixel::Rail, path_rail_game_point);
            }
        }
    }
    println!(
        "metric successors called {}",
        METRIC_SUCCESSOR.get().to_formatted_string(&LOCALE)
    )
}
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RailPoint {
    x: i32,
    y: i32,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RailMode {
    Straight,
    Curved,
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
    pub area: Vec<RailPoint>,
    pub direction: RailDirection,
    pub mode: RailMode,
}

const DUAL_RAIL_SIZE: u32 = 3;

const SAFETY_ZERO: Cell<bool> = Cell::new(false);
const METRIC_SUCCESSOR: Cell<u64> = Cell::new(0);

impl Rail {
    pub fn new_straight(point: PointU32, direction: RailDirection) -> Self {
        SAFETY_ZERO.set(true);
        let mut res = Rail {
            endpoint: RailPoint::from_point_u32(point).round(),
            area: Vec::new(),
            mode: RailMode::Straight,
            direction,
        };
        res.area = res.build_dual_rail_area();
        SAFETY_ZERO.set(false);

        res
    }

    fn round(&self) -> Self {
        let mut next = self.clone();
        next.endpoint = next.endpoint.round();
        next
    }

    #[allow(dead_code)]
    fn distance(&self, other: &Rail) -> u32 {
        let a = &self.endpoint;
        let b = &other.endpoint;
        a.x.abs_diff(b.x) + a.y.abs_diff(b.y)
    }

    fn move_force_rotate_clockwise(&self, rotations: usize) -> Self {
        if !SAFETY_ZERO.get() && rotations == 0 {
            panic!("0");
        }

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

    fn move_forward(&self) -> Option<Self> {
        let mut next = self.clone();
        next.mode = RailMode::Straight;

        next.move_force_forward_mut(6);
        if next.endpoint.is_negative() {
            None
        } else {
            next.area = next.build_dual_rail_area();
            Some(next)
        }
    }

    fn build_dual_rail_area(&self) -> Vec<RailPoint> {
        let next = self.move_force_rotate_clockwise(1);
        (0..DUAL_RAIL_SIZE)
            .map(|i| next.move_force_forward(i).endpoint)
            .collect()
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
        next.move_force_forward_mut(6);
        next = next.move_force_rotate_clockwise(rotation_steps);
        next.move_force_forward_mut(6);

        if next.endpoint.is_negative() {
            None
        } else {
            next.mode = RailMode::Curved;

            next.area = self.build_dual_rail_area();
            // turn area build start is previous position
            next.area.extend(self.build_turn_area(&next.direction));

            // let before = next.area.len();
            // next.area.dedup();
            // let after = next.area.len();
            // if before != after {
            //     panic!("broken area check {} \n\n {}", before, after);
            // }

            Some(next)
        }
    }

    fn build_turn_area(&self, new_direction: &RailDirection) -> Vec<RailPoint> {
        let mut check_rail = Vec::new();
        let mut start = self.clone();

        start.move_force_forward_mut(1);

        for width in 0..(6 + DUAL_RAIL_SIZE) {
            for height in 0..(6 + DUAL_RAIL_SIZE) {
                let mut next = start.clone();
                next.move_force_forward_mut(width);
                next.direction = new_direction.clone();
                next.move_force_forward_mut(height);

                check_rail.push(next.endpoint);
            }
        }
        check_rail
    }

    fn successors(&self, surface: &Surface, end: &Rail) -> Vec<(Self, u32)> {
        METRIC_SUCCESSOR.update(|v| v + 1);
        let direction_bias = if self.direction != end.direction {
            100
        } else {
            0
        };

        let mut res = Vec::new();
        if let Some(rail) = self.move_forward(7).and_then(|v| v.into_buildable(surface)) {
            res.push((rail, 2 + direction_bias))
        }
        if let Some(rail) = self.move_left().and_then(|v| v.into_buildable(surface)) {
            res.push((rail, 1000))
        }
        if let Some(rail) = self.move_right().and_then(|v| v.into_buildable(surface)) {
            res.push((rail, 1000))
        }
        // println!(
        //     "for {:?} found {}",
        //     &self,
        //     res.iter().map(|(rail, _)| format!("{:?}", rail)).join("|")
        // );
        res
    }
    fn into_buildable(self, surface: &Surface) -> Option<Self> {
        self.area
            .iter()
            .map(|r| r.to_game_points())
            .map(|game_points| {
                if game_points.len() == 4 {
                    Some(game_points)
                } else {
                    None
                }
            })
            .filter_map(|v| v)
            .flatten()
            .map(|game_pont| is_buildable_point_u32(surface, game_pont))
            .reduce(|total, is_buildable| total.and(is_buildable))
            .unwrap()
            .map(|_| self)
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

fn is_buildable_point_u32(surface: &Surface, point: PointU32) -> Option<PointU32> {
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

#[cfg(test)]
mod test {
    use crate::navigator::devo::{devo_start, Rail, RailDirection};
    use crate::surface::pixel::Pixel;
    use crate::surface::surface::{PointU32, Surface};
    use std::path::Path;

    #[test]
    fn spawn_function() {
        let mut surface = Surface::new(100, 100);
        // surface.set_pixel(Pixel::IronOre, 50, 5);
        devo_start(
            &mut surface,
            Rail::new_straight(PointU32 { x: 15, y: 15 }, RailDirection::Right),
            Rail::new_straight(PointU32 { x: 85, y: 15 }, RailDirection::Right),
        );
        surface.save(Path::new("work/test"))
    }

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
