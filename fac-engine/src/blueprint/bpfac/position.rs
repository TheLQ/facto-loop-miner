use serde::{Deserialize, Serialize};

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
}
