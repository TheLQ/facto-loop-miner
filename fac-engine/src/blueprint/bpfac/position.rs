use serde::{Deserialize, Serialize};

use super::BpFacFloat;

#[derive(Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct FacBpPosition {
    x: BpFacFloat,
    y: BpFacFloat,
}

impl FacBpPosition {
    pub fn new(x: BpFacFloat, y: BpFacFloat) -> Self {
        Self { x, y }
    }

    pub fn x(&self) -> BpFacFloat {
        self.x
    }

    pub fn y(&self) -> BpFacFloat {
        self.y
    }
}
