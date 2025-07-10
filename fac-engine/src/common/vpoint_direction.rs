use crate::common::varea::VArea;
use crate::common::vpoint::VPoint;
use crate::game_entities::direction::FacDirectionQuarter;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// aka a Vector
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct VPointDirectionQ(pub VPoint, pub FacDirectionQuarter);

impl VPointDirectionQ {
    pub fn point(&self) -> &VPoint {
        &self.0
    }
    pub fn direction(&self) -> &FacDirectionQuarter {
        &self.1
    }
}

impl Display for VPointDirectionQ {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}-{}", self.0, self.1)
    }
}

#[cfg(test)] // this API is poor outside of tests
impl From<(VPoint, FacDirectionQuarter)> for VPointDirectionQ {
    fn from(value: (VPoint, FacDirectionQuarter)) -> Self {
        Self(value.0, value.1)
    }
}

#[derive(Serialize, Deserialize, Ord, PartialOrd, Clone, Debug, Eq, PartialEq)]
pub struct VSegment {
    pub start: VPointDirectionQ,
    pub end: VPointDirectionQ,
}

impl VSegment {
    // fn assert_step_rails(&self) {
    //     self.start.point().assert_step_rail();
    //     self.end.point().assert_step_rail();
    // }

    pub fn is_within_area(&self, area: &VArea) -> bool {
        area.contains_points_all([self.start.point(), self.end.point()])
    }
}

impl Display for VSegment {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} → {}", self.start, self.end)
    }
}
