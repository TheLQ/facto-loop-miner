use serde::{Deserialize, Serialize};

use super::BpFacFloat;

#[derive(Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BpFacPosition {
    x: BpFacFloat,
    y: BpFacFloat,
}

impl BpFacPosition {
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
