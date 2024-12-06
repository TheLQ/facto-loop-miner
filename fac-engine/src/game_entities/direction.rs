use serde::Deserialize;
use serde::Serialize;
use serde_repr::Deserialize_repr;
use serde_repr::Serialize_repr;
use strum_macros::AsRefStr;
use strum_macros::IntoStaticStr;

#[derive(Debug, Clone, PartialEq, AsRefStr, IntoStaticStr, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum FacDirectionEighth {
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
}

#[derive(Debug, Clone, AsRefStr, IntoStaticStr)]
pub enum FacDirectionQuarter {
    North,
    East,
    South,
    West,
}

const RAIL_DIRECTION_CLOCKWISE: [RailDirection; 4] = [
    RailDirection::Up,
    RailDirection::Right,
    RailDirection::Down,
    RailDirection::Left,
];

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

    pub fn is_up_down(&self) -> bool {
        *self == RailDirection::Up || *self == RailDirection::Down
    }
}
