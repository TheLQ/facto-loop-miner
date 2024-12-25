use crate::common::vpoint::VPoint;

pub trait FacBlock {
    fn generate(&self, origin: VPoint);
}
