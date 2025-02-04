use serde::{Deserialize, Serialize};
use strum::AsRefStr;

use crate::common::{
    entity::{FacEntity, SquareArea},
    names::FacEntityName,
};

use super::{
    belt::FacEntBeltType,
    direction::{FacDirectionEighth, FacDirectionQuarter},
};

#[derive(Debug, Clone, Copy, PartialEq, AsRefStr, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FacEntBeltUnderType {
    Input,
    Output,
}

impl FacEntBeltUnderType {
    pub fn flip(&self) -> Self {
        match self {
            Self::Input => Self::Output,
            Self::Output => Self::Input,
        }
    }
}

impl FacEntBeltUnderType {
    /// needed for lua
    pub fn to_fac(&self) -> String {
        self.as_ref().to_lowercase()
    }
}

#[derive(Debug)]
pub struct FacEntBeltUnder {
    btype: FacEntBeltType,
    direction: FacDirectionQuarter,
    utype: FacEntBeltUnderType,
}

impl FacEntity for FacEntBeltUnder {
    fn name(&self) -> FacEntityName {
        FacEntityName::BeltUnder(self.btype)
    }

    fn to_fac_direction(&self) -> Option<FacDirectionEighth> {
        Some(self.direction.to_direction_eighth())
    }

    fn to_fac_belt_under_type(&self) -> Option<FacEntBeltUnderType> {
        Some(self.utype.clone())
    }
}

impl SquareArea for FacEntBeltUnder {
    fn area_diameter() -> usize {
        1
    }
}

impl FacEntBeltUnder {
    pub fn new(
        btype: FacEntBeltType,
        direction: FacDirectionQuarter,
        utype: FacEntBeltUnderType,
    ) -> Self {
        Self {
            btype,
            direction,
            utype,
        }
    }
}
