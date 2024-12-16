use serde_repr::Deserialize_repr;
use serde_repr::Serialize_repr;
use strum::AsRefStr;
use strum::IntoStaticStr;

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
#[repr(u8)]
pub enum FacDirectionQuarter {
    // clockwise order
    North,
    East,
    South,
    West,
}

impl FacDirectionQuarter {
    pub const fn to_direction_eighth(&self) -> FacDirectionEighth {
        match self {
            Self::North => FacDirectionEighth::North,
            Self::East => FacDirectionEighth::East,
            Self::South => FacDirectionEighth::South,
            Self::West => FacDirectionEighth::West,
        }
    }

    pub const fn rotate_flip(&self) -> FacDirectionQuarter {
        match self {
            Self::North => Self::South,
            Self::East => Self::West,
            Self::South => Self::North,
            Self::West => Self::East,
        }
    }

    pub fn rotate_once(&self) -> FacDirectionQuarter {
        match self {
            Self::North => Self::East,
            Self::East => Self::South,
            Self::South => Self::West,
            Self::West => Self::North,
        }
    }

    // pub fn is_up_down(&self) -> bool {
    //     *self == RailDirection::Up || *self == RailDirection::Down
    // }

    // pub fn spinner(&self, post_rotations: usize) -> RailDirection {
    //     let mut directions = RAIL_DIRECTION_CLOCKWISE.iter().cycle();

    //     while directions.next().unwrap() != self {}

    //     let mut new_direction = directions.next().unwrap();
    //     for _ in 1..post_rotations {
    //         new_direction = directions.next().unwrap();
    //     }
    //     new_direction.clone()
    // }

    // pub fn is_same_axis(&self, other: &Self) -> bool {
    //     match self {
    //         RailDirection::Up | RailDirection::Down => {
    //             *other == RailDirection::Up || *other == RailDirection::Down
    //         }
    //         RailDirection::Left | RailDirection::Right => {
    //             *other == RailDirection::Left || *other == RailDirection::Right
    //         }
    //     }
    // }
}
