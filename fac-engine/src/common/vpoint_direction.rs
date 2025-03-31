use crate::common::vpoint::VPoint;
use crate::game_entities::direction::FacDirectionQuarter;
use std::fmt::{Display, Formatter};

/// aka a Vector
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
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
