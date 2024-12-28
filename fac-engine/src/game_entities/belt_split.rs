use serde::{Deserialize, Serialize};

use crate::common::{
    entity::{FacArea, FacEntity, Size},
    names::FacEntityName,
};

use super::{
    belt::FacEntBeltType,
    direction::{FacDirectionEighth, FacDirectionQuarter},
};

#[derive(Debug)]
pub struct FacEntBeltSplit {
    name: FacEntityName,
    direction: FacDirectionQuarter,
    priority: Option<FacEntBeltSplitPriority>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct FacEntBeltSplitPriority {
    pub input: FacExtPriority,
    pub output: FacExtPriority,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FacExtPriority {
    Left,
    Right,
    None,
}

impl FacExtPriority {
    pub fn is_none(&self) -> bool {
        self == &Self::None
    }
}

impl Default for FacExtPriority {
    fn default() -> Self {
        Self::None
    }
}

impl FacEntity for FacEntBeltSplit {
    fn name(&self) -> &FacEntityName {
        &self.name
    }

    fn to_fac_direction(&self) -> Option<FacDirectionEighth> {
        Some(self.direction.to_direction_eighth())
    }

    fn to_fac_input_priority(&self) -> FacExtPriority {
        self.priority.as_ref().map(|v| v.input).unwrap_or_default()
    }

    fn to_fac_output_priority(&self) -> FacExtPriority {
        self.priority.as_ref().map(|v| v.output).unwrap_or_default()
    }
}

impl FacArea for FacEntBeltSplit {
    fn rectangle_size(&self) -> Size {
        match self.direction {
            FacDirectionQuarter::North | FacDirectionQuarter::South => Size::rectangle(2, 1),
            FacDirectionQuarter::East | FacDirectionQuarter::West => Size::rectangle(1, 2),
        }
    }
}

impl FacEntBeltSplit {
    pub fn new(btype: FacEntBeltType, direction: FacDirectionQuarter) -> Self {
        Self {
            name: FacEntityName::BeltSplit(btype),
            direction,
            priority: None,
        }
    }

    pub fn new_priority(
        btype: FacEntBeltType,
        direction: FacDirectionQuarter,
        priority: FacEntBeltSplitPriority,
    ) -> Self {
        Self {
            name: FacEntityName::BeltSplit(btype),
            direction,
            priority: Some(priority),
        }
    }
}
