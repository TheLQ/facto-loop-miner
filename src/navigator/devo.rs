use crate::surface::pixel::Pixel;
use crate::surface::surface::Surface;
use itertools::Itertools;
use pathfinding::prelude::astar;

pub fn devo_start(surface: &mut Surface, start: Rail, end: Rail) {
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
    fn distance(&self, other: &Rail) -> u32 {
        self.x.abs_diff(other.x) + self.y.abs_diff(other.y)
    }

    fn move_force_rotate_clockwise(&self, rotations: usize) -> Rail {
        if rotations == 0 {
            panic!("0");
        }

        let mut directions = RAIL_DIRECTION_CLOCKWISE.iter().cycle();

        while directions.next().unwrap() != &self.direction {}

        let mut new_direction = directions.next().unwrap();
        for _ in 1..rotations {
            new_direction = directions.next().unwrap();
        }

        Rail {
            x: self.x,
            y: self.y,
            direction: new_direction.clone(),
        }
    }

    fn move_forward(&self, steps: u32) -> Rail {
        let mut next = Rail {
            x: self.x,
            y: self.y,
            direction: self.direction.clone(),
        };
        match self.direction {
            RailDirection::Up => next.y = next.y + steps,
            RailDirection::Down => next.y = next.y - steps,
            RailDirection::Left => next.x = next.x - steps,
            RailDirection::Right => next.x = next.x + steps,
        };
        next
    }

    fn move_left(&self) -> Rail {
        // let mut next = self.move_forward(12);
        // next = next.move_force_rotate_clockwise(1);
        // next = next.move_forward(12);
        let mut next = self.move_forward(1);
        next = next.move_force_rotate_clockwise(1);
        next = next.move_forward(1);
        next
    }

    fn move_right(&self) -> Rail {
        // let mut next = self.move_forward(12);
        // next = next.move_force_rotate_clockwise(3);
        // next = next.move_forward(12);
        let mut next = self.move_forward(1);
        next = next.move_force_rotate_clockwise(3);
        next = next.move_forward(1);
        next
    }

    fn successors(&self, surface: &Surface) -> Vec<(Rail, u32)> {
        let mut res = Vec::new();
        if let Some(rail) = is_position_buildable(surface, self.move_forward(1)) {
            res.push((rail, 1))
        }
        if let Some(rail) = is_position_buildable(surface, self.move_left()) {
            res.push((rail, 3))
        }
        if let Some(rail) = is_position_buildable(surface, self.move_right()) {
            res.push((rail, 3))
        }
        // println!(
        //     "for {:?} found {}",
        //     &self,
        //     res.iter().map(|(rail, _)| format!("{:?}", rail)).join("|")
        // );
        res
    }
}

fn is_position_buildable(surface: &Surface, position: Rail) -> Option<Rail> {
    if !surface.xy_in_range(position.x, position.y) {
        return None;
    }
    match surface.get_pixel(position.x, position.y) {
        Pixel::Empty | Pixel::Highlighter | Pixel::EdgeWall => Some(position),
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
