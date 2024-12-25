use crate::{
    blueprint::output::FacItemOutput, common::vpoint::VPoint,
};

pub trait FacBlock {
    fn generate(&self, origin: VPoint, ouput: &mut FacItemOutput);
}
