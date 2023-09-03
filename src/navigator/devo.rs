use crate::surface::pixel::Pixel;
use crate::surface::surface::{PointU32, Surface};
use itertools::Itertools;
use num_format::Locale::is;
use pathfinding::prelude::astar;

pub fn devo_start(surface: &mut Surface, mut start: Rail, mut end: Rail) {
    start = start.round();
    end = end.round();
    println!("Devo start {:?} end {:?}", start, end);
    let (path, path_size) =
        astar(&start, |p| p.successors(surface), |_p| 1, |p| *p == end).unwrap();
    println!("built path {} long with {}", path.len(), path_size);
    let mut total_rail = 0;
    for entity in path {
        for game_entity in entity.to_game_points_3_wide() {
            total_rail = total_rail + 1;
            if surface.get_pixel_point_u32(game_entity) == &Pixel::Rail {
                panic!("existing {:?} at {}", game_entity, total_rail)
            }
            surface.set_pixel_point_u32(Pixel::Rail, game_entity);
        }
    }
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
    pub x: u32,
    pub y: u32,
    pub direction: RailDirection,
}

impl Rail {
    fn round(&self) -> Self {
        let mut next = self.clone();
        next.x = next.x + (self.x % 2);
        next.y = next.y + (self.y % 2);
        next
    }

    fn to_point_u32(&self) -> PointU32 {
        PointU32 {
            x: self.x,
            y: self.y,
        }
    }

    fn distance(&self, other: &Rail) -> u32 {
        self.x.abs_diff(other.x) + self.y.abs_diff(other.y)
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
        // rail is 2x2
        let steps = steps * 2;
        match self.direction {
            RailDirection::Up => next.y = next.y + steps,
            RailDirection::Down => next.y = next.y - steps,
            RailDirection::Left => next.x = next.x - steps,
            RailDirection::Right => next.x = next.x + steps,
        };
        next
    }

    fn move_left(&self) -> Self {
        let mut next = self.move_forward(6);
        next = next.move_force_rotate_clockwise(1);
        next = next.move_forward(6);
        next
    }

    fn move_right(&self) -> Self {
        let mut next = self.move_forward(6);
        next = next.move_force_rotate_clockwise(3);
        next = next.move_forward(6);
        next
    }

    fn move_force_adjacent(&self) -> Self {
        let orig = self.clone();

        let mut next = self.move_force_rotate_clockwise(1);
        next = next.move_forward(1);
        // recover original direction
        next = next.move_force_rotate_clockwise(3);

        if orig == next {
            panic!("didn't do anything? {:?}", orig);
        } else if orig.direction != next.direction {
            panic!("wrong dir {:?}", next.direction.clone());
        }

        next
    }

    fn successors(&self, surface: &Surface) -> Vec<(Self, u32)> {
        let mut res = Vec::new();
        if let Some(rail) = is_dual_rail_buildable(surface, self.move_forward(1)) {
            res.push((rail, 2))
        }
        if let Some(rail) = is_dual_rail_buildable(surface, self.move_left()) {
            res.push((rail, 1000))
        }
        if let Some(rail) = is_dual_rail_buildable(surface, self.move_right()) {
            res.push((rail, 1000))
        }
        // println!(
        //     "for {:?} found {}",
        //     &self,
        //     res.iter().map(|(rail, _)| format!("{:?}", rail)).join("|")
        // );
        res
    }

    fn to_game_points(&self) -> [PointU32; 4] {
        [
            self.to_point_u32(),
            {
                let mut v = self.to_point_u32();
                v.x = v.x - 1;
                v
            },
            {
                let mut v = self.to_point_u32();
                v.y = v.y - 1;
                v
            },
            {
                let mut v = self.to_point_u32();
                v.x = v.x - 1;
                v.y = v.y - 1;
                v
            },
        ]
    }

    fn to_game_points_3_wide(&self) -> impl Iterator<Item = PointU32> {
        [
            self.to_game_points(),
            self.move_force_adjacent().to_game_points(),
            self.move_force_adjacent()
                .move_force_adjacent()
                .to_game_points(),
        ]
        .into_iter()
        .flatten()
    }
}

fn is_dual_rail_buildable(surface: &Surface, rail: Rail) -> Option<Rail> {
    rail.to_game_points_3_wide()
        .map(|game_pont| is_buildable_point_u32(surface, game_pont))
        .into_iter()
        .reduce(|total, is_buildable| total.and(is_buildable))
        .unwrap()
        .map(|_| rail)
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
    use crate::navigator::devo::{Rail, RailDirection};

    #[test]
    fn it_works() {
        println!("canary2");

        let mut rail = Rail {
            x: 0,
            y: 0,
            // second to last
            direction: RailDirection::Down,
        };
        rail = rail.move_force_rotate_clockwise(1);
        assert_eq!(rail.direction, RailDirection::Left, "rotation");
        rail = rail.move_force_rotate_clockwise(2);
        assert_eq!(rail.direction, RailDirection::Right);
    }
}
