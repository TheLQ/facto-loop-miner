use serde::Deserialize;
use serde::Serialize;
use strum::IntoStaticStr;
use strum::VariantArray;
use strum::{AsRefStr, Display};

#[derive(
    Debug, Clone, PartialEq, AsRefStr, IntoStaticStr, VariantArray, Serialize, Deserialize,
)]
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

impl FacDirectionEighth {
    pub const fn rotate_once(&self) -> FacDirectionEighth {
        match self {
            Self::North => Self::NorthEast,
            Self::NorthEast => Self::East,
            Self::East => Self::SouthEast,
            Self::SouthEast => Self::South,
            Self::South => Self::SouthWest,
            Self::SouthWest => Self::West,
            Self::West => Self::NorthWest,
            Self::NorthWest => Self::North,
        }
    }

    pub const fn rotate_opposite(&self) -> FacDirectionEighth {
        match self {
            Self::North => Self::NorthWest,
            Self::NorthEast => Self::North,
            Self::East => Self::NorthEast,
            Self::SouthEast => Self::East,
            Self::South => Self::SouthEast,
            Self::SouthWest => Self::South,
            Self::West => Self::SouthWest,
            Self::NorthWest => Self::West,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Display, AsRefStr, IntoStaticStr, VariantArray)]
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

    pub const fn rotate_once(&self) -> FacDirectionQuarter {
        match self {
            Self::North => Self::East,
            Self::East => Self::South,
            Self::South => Self::West,
            Self::West => Self::North,
        }
    }

    pub const fn rotate_opposite(&self) -> FacDirectionQuarter {
        match self {
            Self::North => Self::West,
            Self::East => Self::North,
            Self::South => Self::East,
            Self::West => Self::South,
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

#[cfg(test)]
mod test {
    use strum::VariantArray;

    use super::{FacDirectionEighth, FacDirectionQuarter};

    fn direction_quarter_get(i: usize) -> &'static FacDirectionQuarter {
        &FacDirectionQuarter::VARIANTS[i % 4]
    }

    fn direction_quarter_index_of(direction: &FacDirectionQuarter) -> usize {
        FacDirectionQuarter::VARIANTS
            .iter()
            .position(|v| v == direction)
            .unwrap()
    }

    #[test]
    fn test_quarter_rotate_flip() {
        for direction in FacDirectionQuarter::VARIANTS {
            assert_eq!(
                &direction.rotate_flip(),
                direction_quarter_get(direction_quarter_index_of(direction) + 2)
            )
        }
    }

    #[test]
    fn test_quarter_rotate_once() {
        for direction in FacDirectionQuarter::VARIANTS {
            assert_eq!(
                &direction.rotate_once(),
                direction_quarter_get(direction_quarter_index_of(direction) + 1)
            )
        }
    }

    #[test]
    fn test_quarter_rotate_opposite() {
        for direction in FacDirectionQuarter::VARIANTS {
            assert_eq!(
                &direction.rotate_opposite(),
                direction_quarter_get(direction_quarter_index_of(direction) + 3)
            )
        }
    }

    fn direction_eighth_get(i: usize) -> &'static FacDirectionEighth {
        &FacDirectionEighth::VARIANTS[i % 8]
    }

    fn direction_eighth_index_of(direction: &FacDirectionEighth) -> usize {
        FacDirectionEighth::VARIANTS
            .iter()
            .position(|v| v == direction)
            .unwrap()
    }

    // #[test]
    // fn test_eighth_rotate_flip() {
    //     for direction in FacDirectionEighth::VARIANTS {
    //         assert_eq!(
    //             &direction.rotate_flip(),
    //             direction_quarter_get(direction_eighth_index_of(direction) + 2)
    //         )
    //     }
    // }

    #[test]
    fn test_eighth_rotate_once() {
        for direction in FacDirectionEighth::VARIANTS {
            assert_eq!(
                &direction.rotate_once(),
                direction_eighth_get(direction_eighth_index_of(direction) + 1),
                "from source dir {:?}",
                direction,
            )
        }
    }

    #[test]
    fn test_eighth_rotate_opposite() {
        for direction in FacDirectionEighth::VARIANTS {
            assert_eq!(
                &direction.rotate_opposite(),
                direction_eighth_get(direction_eighth_index_of(direction) + 7),
                "from source dir {:?}",
                direction,
            )
        }
    }
}
