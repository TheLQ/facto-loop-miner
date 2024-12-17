use serde_repr::Deserialize_repr;
use serde_repr::Serialize_repr;
use strum::AsRefStr;
use strum::IntoStaticStr;
use strum::VariantArray;

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

#[derive(Debug, Clone, PartialEq, AsRefStr, IntoStaticStr, VariantArray)]
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

    use super::FacDirectionQuarter;

    const DIRECTION_ORDER: [FacDirectionQuarter; 4] = [
        FacDirectionQuarter::North,
        FacDirectionQuarter::East,
        FacDirectionQuarter::South,
        FacDirectionQuarter::West,
    ];

    fn direction_get(i: usize) -> &'static FacDirectionQuarter {
        &DIRECTION_ORDER[i % 4]
    }

    fn direction_index_of(direction: &FacDirectionQuarter) -> usize {
        DIRECTION_ORDER.iter().position(|v| v == direction).unwrap()
    }

    #[test]
    fn test_rotate_flip() {
        for direction in FacDirectionQuarter::VARIANTS {
            assert_eq!(
                &direction.rotate_flip(),
                direction_get(direction_index_of(direction) + 2)
            )
        }
    }

    #[test]
    fn test_rotate_once() {
        for direction in FacDirectionQuarter::VARIANTS {
            assert_eq!(
                &direction.rotate_once(),
                direction_get(direction_index_of(direction) + 1)
            )
        }
    }

    #[test]
    fn test_rotate_opposite() {
        for direction in FacDirectionQuarter::VARIANTS {
            assert_eq!(
                &direction.rotate_opposite(),
                direction_get(direction_index_of(direction) + 3)
            )
        }
    }
}
