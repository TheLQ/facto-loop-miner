use crate::{blueprint::bpitem::BlueprintItem, common::vpoint::VPoint};

pub trait BlockFac {
    fn generate(&self, origin: VPoint, results: &mut Vec<BlueprintItem>);
}
