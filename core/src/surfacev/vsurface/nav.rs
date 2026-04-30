use crate::surfacev::mine::{MineLocation, MinePath};
use crate::surfacev::ventity_map::{VEntityMap, VPixel};
use crate::surfacev::vpatch::VPatch;

pub struct PlugMut<'s> {
    pub(super) pixels: &'s mut VEntityMap<VPixel>,
    pub(super) patches: &'s mut Vec<VPatch>,
    pub(super) mines: &'s mut Vec<MineLocation>,
    pub(super) rails: &'s mut Vec<MinePath>,
}

#[derive(Clone, Copy)]
pub struct Plug<'s> {
    pub(super) pixels: &'s VEntityMap<VPixel>,
    pub(super) patches: &'s Vec<VPatch>,
    pub(super) mines: &'s Vec<MineLocation>,
    pub(super) rails: &'s Vec<MinePath>,
}

impl Plug<'_> {}

//

pub trait AsVsMut {
    //: AsVs<'s> {
    fn nav_mut(&mut self) -> PlugMut<'_>;
}

pub trait AsVs {
    fn nav(&self) -> Plug<'_>;
}
