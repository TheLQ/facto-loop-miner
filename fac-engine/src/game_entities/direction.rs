use serde::Deserialize;
use serde::Serialize;
use serde_repr::Deserialize_repr;
use serde_repr::Serialize_repr;
use strum::IntoStaticStr;
use strum::VariantArray;
use strum::{AsRefStr, Display};

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    AsRefStr,
    IntoStaticStr,
    VariantArray,
    Serialize_repr,
    Deserialize_repr,
)]
// repr(u8) in order of https://lua-api.factorio.com/1.1.110/defines.html#defines.direction
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

impl FacDirectionEighth {
    #[cfg(test)]
    fn get_index(i: usize) -> &'static Self {
        &Self::VARIANTS[i % Self::VARIANTS.len()]
    }

    #[cfg(test)]
    fn index_of(direction: &Self) -> usize {
        Self::VARIANTS.iter().position(|v| v == direction).unwrap()
    }

    pub const fn rotate_flip(&self) -> Self {
        match self {
            Self::North => Self::South,
            Self::NorthEast => Self::SouthWest,
            Self::East => Self::West,
            Self::SouthEast => Self::NorthWest,
            Self::South => Self::North,
            Self::SouthWest => Self::NorthEast,
            Self::West => Self::East,
            Self::NorthWest => Self::SouthEast,
        }
    }

    pub const fn rotate_once(&self) -> Self {
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

    pub const fn rotate_opposite(&self) -> Self {
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

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Display,
    AsRefStr,
    IntoStaticStr,
    VariantArray,
    Serialize,
    Deserialize,
)]
#[repr(u8)]
pub enum FacDirectionQuarter {
    // clockwise order
    North,
    East,
    South,
    West,
}

impl FacDirectionQuarter {
    #[cfg(test)]
    fn get_index(i: usize) -> &'static Self {
        &Self::VARIANTS[i % Self::VARIANTS.len()]
    }

    #[cfg(test)]
    fn index_of(direction: &Self) -> usize {
        Self::VARIANTS.iter().position(|v| v == direction).unwrap()
    }

    pub const fn to_direction_eighth(&self) -> FacDirectionEighth {
        match self {
            Self::North => FacDirectionEighth::North,
            Self::East => FacDirectionEighth::East,
            Self::South => FacDirectionEighth::South,
            Self::West => FacDirectionEighth::West,
        }
    }

    pub const fn rotate_flip(&self) -> Self {
        match self {
            Self::North => Self::South,
            Self::East => Self::West,
            Self::South => Self::North,
            Self::West => Self::East,
        }
    }

    pub const fn rotate_once(&self) -> Self {
        match self {
            Self::North => Self::East,
            Self::East => Self::South,
            Self::South => Self::West,
            Self::West => Self::North,
        }
    }

    pub const fn rotate_opposite(&self) -> Self {
        match self {
            Self::North => Self::West,
            Self::East => Self::North,
            Self::South => Self::East,
            Self::West => Self::South,
        }
    }

    pub const fn rotate_clockwise(&self, clockwise: bool) -> Self {
        if clockwise {
            self.rotate_once()
        } else {
            self.rotate_opposite()
        }
    }

    // pub const fn rotate_once_if(&self, enabled: bool) -> Self {
    //     if enabled { self.rotate_once() } else { *self }
    // }

    pub fn as_sign_f32(&self) -> f32 {
        match &self {
            FacDirectionQuarter::North | FacDirectionQuarter::West => -1.0,
            FacDirectionQuarter::South | FacDirectionQuarter::East => 1.0,
        }
    }

    pub fn is_up_down(&self) -> bool {
        match self {
            Self::North | Self::South => true,
            Self::East | Self::West => false,
        }
    }

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

    #[test]
    fn test_quarter_rotate_flip() {
        for direction in FacDirectionQuarter::VARIANTS {
            assert_eq!(
                &direction.rotate_flip(),
                FacDirectionQuarter::get_index(FacDirectionQuarter::index_of(direction) + 2)
            )
        }
    }

    #[test]
    fn test_quarter_rotate_once() {
        for direction in FacDirectionQuarter::VARIANTS {
            assert_eq!(
                &direction.rotate_once(),
                FacDirectionQuarter::get_index(FacDirectionQuarter::index_of(direction) + 1)
            )
        }
    }

    #[test]
    fn test_quarter_rotate_opposite() {
        for direction in FacDirectionQuarter::VARIANTS {
            assert_eq!(
                &direction.rotate_opposite(),
                FacDirectionQuarter::get_index(FacDirectionQuarter::index_of(direction) + 3)
            )
        }
    }

    #[test]
    fn test_eighth_rotate_flip() {
        for direction in FacDirectionEighth::VARIANTS {
            assert_eq!(
                &direction.rotate_flip(),
                FacDirectionEighth::get_index(FacDirectionEighth::index_of(direction) + 4),
                "from source dir {:?}",
                direction,
            )
        }
    }

    #[test]
    fn test_eighth_rotate_once() {
        for direction in FacDirectionEighth::VARIANTS {
            assert_eq!(
                &direction.rotate_once(),
                FacDirectionEighth::get_index(FacDirectionEighth::index_of(direction) + 1),
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
                FacDirectionEighth::get_index(FacDirectionEighth::index_of(direction) + 7),
                "from source dir {:?}",
                direction,
            )
        }
    }
}
