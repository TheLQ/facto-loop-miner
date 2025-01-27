use crate::common::vpoint::VPoint;
use crate::game_entities::direction::FacDirectionQuarter;
use std::fmt::{Display, Formatter};

/// aka a Vector
#[derive(Clone)]
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
