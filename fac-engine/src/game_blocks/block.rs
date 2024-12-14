use crate::{blueprint::bpitem::BlueprintItem, common::vpoint::VPoint};

pub trait FacBlock {
    fn generate(&self, origin: VPoint) -> Vec<BlueprintItem>;
}
