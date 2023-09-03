use crate::surface::pixel::Pixel;
use crate::surface::surface::{PointU32, Surface};
use itertools::Itertools;
use num_format::Locale::{is, se};
use opencv::core::Point;
use pathfinding::prelude::astar;

pub fn devo_start(surface: &mut Surface, mut start: Rail, mut end: Rail) {
    start.mode = start.mode.round();
    end.mode = end.mode.round();
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
        for path_rail_point in path_rail.area_points() {
            for path_rail_game_point in path_rail_point.to_game_points() {
                total_rail = total_rail + 1;
                if surface.get_pixel_point_u32(path_rail_game_point) == &Pixel::Rail {
                    panic!("existing {:?} at {}", path_rail_point, total_rail)
                }
                surface.set_pixel_point_u32(Pixel::Rail, path_rail_game_point);
            }
        }
    }
}
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FactoPointU32 {
    x: u32,
    y: u32,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RailMode {
    Straight(FactoPointU32, Vec<FactoPointU32>),
    Curved(FactoPointU32, Vec<FactoPointU32>),
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
    pub direction: RailDirection,
    pub mode: RailMode,
}

const DUAL_RAIL_SIZE: u32 = 3;

impl Rail {
    pub fn new_straight(point: PointU32, direction: RailDirection) -> Self {
        let mut res = Rail {
            mode: RailMode::Straight(FactoPointU32::from_point_u32(point), Vec::new()),
            direction,
        };

        // res.mode = RailMode::Straight(
        //     res.mode.endpoint(),
        //     res.move_force_adjacent(DUAL_RAIL_SIZE as u32 - 1)
        //         .into_iter()
        //         .map(|rail| rail.mode.endpoint())
        //         .collect(),
        // );

        res
    }

    fn area_points(&self) -> Vec<&FactoPointU32> {
        let mut res = Vec::new();
        match &self.mode {
            RailMode::Straight(end, adjacent_box) => {
                res.push(end);
                for turn_point in adjacent_box {
                    res.push(turn_point);
                }
            }
            RailMode::Curved(end, turn_box) => {
                res.push(end);
                for turn_point in turn_box {
                    res.push(turn_point);
                }
            }
        }
        res
    }

    fn distance(&self, other: &Rail) -> u32 {
        let a = self.mode.endpoint();
        let b = other.mode.endpoint();
        a.x.abs_diff(b.x) + a.y.abs_diff(b.y)
    }

    fn move_force_rotate_clockwise(&self, rotations: usize) -> Self {
        if rotations == 0 {
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

    fn move_forward(&self, steps: u32) -> Self {
        if steps == 0 {
            panic!("0");
        }
        let mut next = self.clone();
        let mut next_endpoint = next.mode.endpoint();
        // rail is 2x2
        let steps = steps * 2;
        match self.direction {
            RailDirection::Up => next_endpoint.y = next_endpoint.y + steps,
            RailDirection::Down => next_endpoint.y = next_endpoint.y - steps,
            RailDirection::Left => next_endpoint.x = next_endpoint.x - steps,
            RailDirection::Right => next_endpoint.x = next_endpoint.x + steps,
        };
        next.mode = RailMode::Straight(
            next_endpoint,
            self.move_force_adjacent(DUAL_RAIL_SIZE as u32 - 1)
                .iter()
                .map(|rail| rail.mode.endpoint())
                .collect(),
        );
        next
    }

    fn move_left(&self) -> Self {
        let mut next = self.move_forward(6);
        next = next.move_force_rotate_clockwise(1);
        next = next.move_forward(6);
        next.mode = RailMode::Curved(
            next.mode.endpoint(),
            self.build_turn_box(next.direction.clone()),
        );
        next
    }

    fn move_right(&self) -> Self {
        let mut next = self.move_forward(6);
        next = next.move_force_rotate_clockwise(3);
        next = next.move_forward(6);
        next.mode = RailMode::Curved(
            next.mode.endpoint(),
            self.build_turn_box(next.direction.clone()),
        );
        next
    }

    fn build_turn_box(&self, new_direction: RailDirection) -> Vec<FactoPointU32> {
        let start = self.move_forward(1);
        let mut check_rail = Vec::new();
        for width in 0..(6 + DUAL_RAIL_SIZE) {
            for height in 0..(6 + DUAL_RAIL_SIZE) {
                let mut next = if height != 0 {
                    start.move_forward(height)
                } else {
                    start.clone()
                };
                next.direction = new_direction.clone();
                next = if width != 0 {
                    next.move_forward(width)
                } else {
                    next
                };

                check_rail.push(next.mode.endpoint());
            }
        }
        check_rail
    }

    fn move_force_adjacent(&self, steps: u32) -> Vec<Self> {
        let orig = self.clone();
        let mut res: Vec<Rail> = Vec::new();

        let mut next = self.move_force_rotate_clockwise(1);
        for _ in 0..steps {
            res.push(next.clone());
            next = next.move_forward(1);
        }

        // recover original direction
        next = res.last().unwrap().move_force_rotate_clockwise(3);

        if orig == next {
            panic!("didn't do anything? {:?}", orig);
        } else if orig.direction != next.direction {
            panic!("wrong dir {:?}", next.direction.clone());
        }

        res
    }

    fn successors(&self, surface: &Surface, end: &Rail) -> Vec<(Self, u32)> {
        let direction_bias = if self.direction != end.direction {
            100
        } else {
            0
        };

        let mut res = Vec::new();
        if let Some(rail) = self.move_forward(1).is_buildable(surface) {
            res.push((rail, 2 + direction_bias))
        }
        if let Some(rail) = self.move_left().is_buildable(surface) {
            res.push((rail, 1000))
        }
        if let Some(rail) = self.move_right().is_buildable(surface) {
            res.push((rail, 1000))
        }
        // println!(
        //     "for {:?} found {}",
        //     &self,
        //     res.iter().map(|(rail, _)| format!("{:?}", rail)).join("|")
        // );
        res
    }
    fn is_buildable(&self, surface: &Surface) -> Option<Self> {
        self.area_points()
            .into_iter()
            .map(|r| r.to_game_points())
            .flatten()
            .map(|game_pont| is_buildable_point_u32(surface, game_pont))
            .reduce(|total, is_buildable| total.and(is_buildable))
            .unwrap()
            .map(|_| self.clone())
    }
}

impl FactoPointU32 {
    fn round(&self) -> Self {
        let mut next = self.clone();
        next.x = next.x + (self.x % 2);
        next.y = next.y + (self.y % 2);
        next
    }

    fn to_game_points(&self) -> [PointU32; 4] {
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
        .map(|v| v.to_point_u32())
    }

    fn to_point_u32(&self) -> PointU32 {
        PointU32 {
            x: self.x,
            y: self.y,
        }
    }

    pub fn from_point_u32(point: PointU32) -> Self {
        FactoPointU32 {
            x: point.x,
            y: point.y,
        }
    }
}

impl RailMode {
    fn endpoint(&self) -> FactoPointU32 {
        match self {
            RailMode::Straight(con, _) => con,
            RailMode::Curved(con, _) => con,
        }
        .clone()
    }

    fn round(&self) -> Self {
        let mut next = self.clone();
        next = match next {
            RailMode::Straight(piece, area) => RailMode::Straight(piece.round(), area),
            RailMode::Curved(piece, area) => RailMode::Curved(piece.round(), area),
        };
        next
    }
}

fn is_buildable_point_u32(surface: &Surface, point: PointU32) -> Option<PointU32> {
    if !surface.xy_in_range_point_u32(point) {
        return None;
    }
    match surface.get_pixel_point_u32(point) {
        Pixel::Empty => Some(point),
        _existing => {
            // println!("blocked at {:?} by {:?}", &position, existing);
            None
        }
    }
}
