use crate::navigator::hash_point::HashPoint;
use crate::surface::surface::Surface;
use opencv::core::Point;
use pathfinding::prelude::astar;

pub fn start(surface: &mut Surface, start: Rail, end: Rail) {
    static GOAL: Rail = Rail { x: 0, y: 0 };
    astar(&start, |p| p.successors(), |p| 1, |p| *p == GOAL);
}

enum RailDirection {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct Rail {
    pub x: u32,
    pub y: u32,
    pub direction: RailDirection,
}

impl Rail {
    fn distance(&self, other: &Rail) -> u32 {
        (self.x.abs_diff(other.x) + self.y.abs_diff(other.y))
    }

    fn successors(&self) -> Vec<(Pos, u32)> {
        let &Pos(x, y) = self;
        vec![
            Pos(x + 1, y + 2),
            Pos(x + 1, y - 2),
            Pos(x - 1, y + 2),
            Pos(x - 1, y - 2),
            Pos(x + 2, y + 1),
            Pos(x + 2, y - 1),
            Pos(x - 2, y + 1),
            Pos(x - 2, y - 1),
        ]
        .into_iter()
        .map(|p| (p, 1))
        .collect()
    }
}
