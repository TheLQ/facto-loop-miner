use serde::{Deserialize, Serialize};

use crate::{
    common::vpoint::{VPoint, display_any_pos},
    util::ansi::C_BLOCK_LINE,
};

use super::FacBpFloat;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FacBpPosition {
    pub x: FacBpFloat,
    pub y: FacBpFloat,
}

impl FacBpPosition {
    pub fn new(x: FacBpFloat, y: FacBpFloat) -> Self {
        Self { x, y }
    }

    pub fn x(&self) -> FacBpFloat {
        self.x
    }

    pub fn y(&self) -> FacBpFloat {
        self.y
    }

    pub fn display(&self) -> String {
        display_any_pos(self.x(), self.y())
    }

    pub fn to_vpoint_with_offset(&self, offset_x: f32, offset_y: f32) -> VPoint {
        let new_x = self.x() - offset_x;
        let new_y = self.y() - offset_y;

        if new_x.trunc() != new_x || new_y.trunc() != new_y {
            let new_bppos = FacBpPosition::new(new_x, new_y);
            panic!(
                "not VPoint compatible {} offset {offset_x}{}{offset_y}",
                new_bppos.display(),
                C_BLOCK_LINE
            )
        }
        VPoint::new(new_x as i32, new_y as i32)
    }

    pub fn to_vpoint_exact(&self) -> VPoint {
        self.to_vpoint_with_offset(0.0, 0.0)
    }
}
