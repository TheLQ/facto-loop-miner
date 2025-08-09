use crate::common::vpoint::VPoint;

pub trait FacBlock {
    fn generate(&self, origin: VPoint);
}

pub trait FacBlock2<R> {
    fn generate(&self, origin: VPoint) -> R;
}

pub trait FacBlockFancy<R> {
    fn generate(&self) -> R;
}
