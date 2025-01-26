use crate::common::vpoint::VPoint;
use crate::game_entities::direction::FacDirectionQuarter;

/// aka a Vector
pub struct VPointDirectionQ(pub VPoint, pub FacDirectionQuarter);

impl VPointDirectionQ {
    pub fn point(&self) -> &VPoint {
        &self.0
    }
    pub fn direction(&self) -> &FacDirectionQuarter {
        &self.1
    }
}
