use crate::common::names::FacEntityName;

pub trait FacEntity {
    fn name(&self) -> &FacEntityName;

    fn size(&self) -> &Size;
}

pub struct Size {
    width: u64,
    height: u64,
}

impl Size {
    pub const fn square(size: u64) -> Self {
        Size {
            height: size,
            width: size,
        }
    }
}
