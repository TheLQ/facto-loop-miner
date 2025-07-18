use serde::{Deserialize, Serialize, Serializer};
use std::borrow::Borrow;
use std::fmt::{Display, Formatter};

use crate::{
    common::vpoint::{VPoint, display_any_pos},
    game_entities::direction::FacDirectionQuarter,
};

use super::FacBpFloat;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FacBpPosition {
    #[serde(serialize_with = "serialize_float_without_zero")]
    pub x: FacBpFloat,
    #[serde(serialize_with = "serialize_float_without_zero")]
    pub y: FacBpFloat,
}

impl FacBpPosition {
    pub const fn new(x: FacBpFloat, y: FacBpFloat) -> Self {
        Self { x, y }
    }

    pub const fn x(&self) -> FacBpFloat {
        self.x
    }

    pub const fn y(&self) -> FacBpFloat {
        self.y
    }

    pub const fn move_x(&self, steps: FacBpFloat) -> Self {
        Self {
            x: self.x + steps,
            y: self.y,
        }
    }

    pub const fn move_y(&self, steps: FacBpFloat) -> Self {
        Self {
            x: self.x,
            y: self.y + steps,
        }
    }

    pub fn move_direction(
        &self,
        direction: impl Borrow<FacDirectionQuarter>,
        steps: FacBpFloat,
    ) -> Self {
        // cardinal directions are "north == up == -1" not "north == +1"
        match direction.borrow() {
            FacDirectionQuarter::North => self.move_y(-steps),
            FacDirectionQuarter::South => self.move_y(steps),
            FacDirectionQuarter::East => self.move_x(steps),
            FacDirectionQuarter::West => self.move_x(-steps),
        }
    }

    pub fn move_direction_and_vpoint_floor(
        &self,
        direction: impl Borrow<FacDirectionQuarter>,
        steps: FacBpFloat,
    ) -> VPoint {
        let new_position = self.move_direction(direction, steps);
        VPoint::new(
            new_position.x().floor() as i32,
            new_position.y().floor() as i32,
        )
    }

    pub fn to_vpoint_with_offset(&self, offset_x: FacBpFloat, offset_y: FacBpFloat) -> VPoint {
        let new_x = self.x() + offset_x;
        let new_y = self.y() + offset_y;

        if new_x.trunc() != new_x || new_y.trunc() != new_y {
            let new_bppos = FacBpPosition::new(new_x, new_y);
            let offset = FacBpPosition::new(offset_x, offset_y);
            panic!("not VPoint compatible {new_bppos} offset {offset}",)
        }
        VPoint::new(new_x as i32, new_y as i32)
    }

    // fn to_vpoint_floor(&self, offset_x: FacBpFloat, offset_y: FacBpFloat) -> VPoint {
    //     let new_x = (self.x() + offset_x).floor() as i32;
    //     let new_y = (self.y() + offset_y).floor() as i32;
    //     VPoint::new(new_x, new_y)
    // }

    pub fn to_vpoint_exact(&self) -> VPoint {
        self.to_vpoint_with_offset(0.0, 0.0)
    }
}

impl Display for FacBpPosition {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let Self { x, y } = self;
        display_any_pos(f, format_args!("{x:>4?}"), format_args!("{y:>4?}"))
    }
}

/// Micro-opt: Factorio does this, which helps reduce every entity's size
/// "1.0" should be "1"
fn serialize_float_without_zero<S>(value: &FacBpFloat, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let value = *value;
    if value.trunc() == value {
        serializer.serialize_i32(value as i32)
    } else {
        serializer.serialize_f32(value)
    }
}

#[cfg(test)]
mod test {
    use crate::blueprint::bpfac::position::FacBpPosition;
    use crate::util::ansi::C_BLOCK_LINE;

    #[test]
    fn display_zero() {
        let pos = FacBpPosition::new(0.0, 0.0);
        assert_eq!(pos.to_string(), format!(" 0.0{C_BLOCK_LINE} 0.0"))
    }

    #[test]
    fn display_ten() {
        let pos = FacBpPosition::new(10.0, 10.0);
        assert_eq!(pos.to_string(), format!("10.0{C_BLOCK_LINE}10.0"))
    }

    #[test]
    fn display_decimal_trunc() {
        let pos = FacBpPosition::new(0.1234567, 0.1234567);
        assert_eq!(pos.to_string(), format!("0.1234567{C_BLOCK_LINE}0.1234567"))
    }
}
