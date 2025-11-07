use crate::surfacev::mine::MinePath;
use crate::surfacev::ventity_map::{VEntityMap, VPixel};
use crate::surfacev::vpatch::VPatch;

pub struct PlugMut<'s> {
    pub(super) rails: &'s mut Vec<MinePath>,
    pub(super) patches: &'s mut Vec<VPatch>,
    pub(super) pixels: &'s mut VEntityMap<VPixel>,
}

#[derive(Clone, Copy)]
pub struct Plug<'s> {
    pub(super) rails: &'s Vec<MinePath>,
    pub(super) patches: &'s Vec<VPatch>,
    pub(super) pixels: &'s VEntityMap<VPixel>,
}

//

pub trait AsVsMut<'s>: AsVs<'s> {
    fn nav_mut(&mut self) -> PlugMut<'s>;
}

pub trait AsVs<'s> {
    fn nav(&'s self) -> Plug<'s>;
}
