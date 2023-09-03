use crate::surface::pixel::Pixel;
use crate::surface::surface::{PointU32, Surface};
use itertools::Itertools;
use pathfinding::prelude::astar;

pub fn devo_start(surface: &mut Surface, mut start: Rail, mut end: Rail) {
    start = Rail {
        x: start.x + (start.x % 2),
        y: start.y + (start.y % 2),
        direction: start.direction,
    };
    end = Rail {
        x: end.x + (end.x % 2),
        y: end.y + (end.y % 2),
        direction: end.direction,
    };
    println!("Devo start {:?} end {:?}", start, end);
    let (path, path_size) =
        astar(&start, |p| p.successors(surface), |_p| 1, |p| *p == end).unwrap();
    println!("built path {} long with {}", path.len(), path_size);
    for entity in path {
        surface.set_pixel(Pixel::Rail, entity.x, entity.y);
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
    fn to_point_u32(&self) -> PointU32 {
        PointU32 {
            x: self.x.clone(),
            y: self.y.clone(),
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

    fn successors(&self, surface: &Surface) -> Vec<(Self, u32)> {
        let mut res = Vec::new();
        if let Some(rail) = is_single_rail_buildable(surface, self.move_forward(1)) {
            res.push((rail, 2))
        }
        if let Some(rail) = is_single_rail_buildable(surface, self.move_left()) {
            res.push((rail, 1000))
        }
        if let Some(rail) = is_single_rail_buildable(surface, self.move_right()) {
            res.push((rail, 1000))
        }
        // println!(
        //     "for {:?} found {}",
        //     &self,
        //     res.iter().map(|(rail, _)| format!("{:?}", rail)).join("|")
        // );
        res
    }
}

fn is_single_rail_buildable(surface: &Surface, position: Rail) -> Option<Rail> {
    is_position_buildable(
        surface,
        Rail {
            x: position.x.clone() - 1,
            y: position.y.clone(),
            direction: position.direction.clone(),
        },
    )
    .and(is_position_buildable(
        surface,
        Rail {
            x: position.x.clone(),
            y: position.y.clone() - 1,
            direction: position.direction.clone(),
        },
    ))
    .and(is_position_buildable(
        surface,
        Rail {
            x: position.x.clone() - 1,
            y: position.y.clone() - 1,
            direction: position.direction.clone(),
        },
    ))
    // Last to give back the original
    .and(is_position_buildable(surface, position.clone()))
}

fn is_position_buildable(surface: &Surface, position: Rail) -> Option<Rail> {
    let point = position.to_point_u32();
    if !surface.xy_in_range_point_u32(point) {
        return None;
    }
    match surface.get_pixel_point_u32(point) {
        Pixel::Empty | Pixel::EdgeWall => Some(position),
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
