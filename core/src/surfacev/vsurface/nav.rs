use crate::surfacev::mine::{MineLocation, MinePath};
use crate::surfacev::ventity_map::{VEntityMap, VPixel};
use crate::surfacev::vpatch::VPatch;
use crate::surfacev::vsurface::MineRef;
use facto_loop_miner_fac_engine::common::vpoint::VPoint;

pub struct PlugMut<'s> {
    pub(super) pixels: &'s mut VEntityMap<VPixel>,
    pub(super) patches: &'s mut Vec<VPatch>,
    pub(super) mines: &'s mut Vec<MineLocation>,
    pub(super) rails: &'s mut Vec<MinePath>,
}

impl PlugMut<'_> {
    pub fn restore_mine_area_buffered(&mut self, mine_ref: MineRef, removed_points: Vec<VPoint>) {
        let mine = &self.mines[mine_ref.0];
        MineLocation::restore_area_buffered(
            &[mine],
            &mut super::pixel::PlugMut {
                pixels: self.pixels,
            },
            removed_points,
        )

        // self.pixels_mut(|s| MineLocation::restore_area_buffered(&[mine], s, removed_points))
    }
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
    fn nav_mut_fn<R>(&mut self, work: impl FnOnce(PlugMut<'_>) -> R) -> R;

    fn nav_mut_old_fn<R>(&mut self, work: impl FnOnce(&mut PlugMut<'_>) -> R) -> R {
        self.nav_mut_fn(|mut s| work(&mut s))
    }

    fn nav_mut_ref(&mut self) -> PlugMut<'_>;
}

pub trait AsVs {
    fn nav(&self) -> Plug<'_>;
}
