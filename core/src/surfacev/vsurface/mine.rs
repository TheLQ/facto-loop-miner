use crate::surfacev::mine::{MineDestination, MineDraw, MineLocation};
use crate::surfacev::ventity_map::{VEntityMap, VPixel};
use crate::surfacev::vsurface::{VSurfaceMine, VSurfacePixelAsVs, VSurfacePixelAsVsMut};
use facto_loop_miner_fac_engine::common::vpoint::VPoint;
use serde::{Deserialize, Serialize};
use simd_json::prelude::ArrayTrait;
use std::borrow::Borrow;

pub struct PlugMut<'s> {
    pub(super) pixels: &'s mut VEntityMap<VPixel>,
    pub(super) mines: &'s mut Vec<MineLocation>,
}

impl PlugMut<'_> {
    pub fn set_mines(&mut self, mines: Vec<MineLocation>) {
        *self.mines = mines;
    }

    pub fn draw_mine(&mut self, input: MineRef, mode: MineDraw) {
        let pixels = &mut self.pixels;
        let mine = &self.mines[input.0];
        mode.draw_mine(&mut super::pixel::PlugMut { pixels }, mine);
    }
}

#[derive(Clone, Copy)]
pub struct Plug<'s> {
    pub(super) pixels: &'s VEntityMap<VPixel>,
    pub(super) mines: &'s Vec<MineLocation>,
}

impl Plug<'_> {
    pub fn get_mine_ref_for(&self, mine: &MineLocation) -> MineRef {
        MineRef(self.mines.iter().position(|v| v == mine).unwrap())
    }

    pub fn get_mines_ref_for<'s>(
        &self,
        mines: impl IntoIterator<Item = &'s MineLocation>,
    ) -> impl Iterator<Item = MineRef> {
        mines.into_iter().map(|mine| self.get_mine_ref_for(mine))
    }

    pub fn resolve_mines(
        &self,
        input: impl IntoIterator<Item = MineRef>,
    ) -> impl Iterator<Item = &MineLocation> {
        input.into_iter().map(|v| v.resolve_mine_surface(*self))
    }

    pub fn resolve_mines_with_refs<I: IntoIterator<Item = MineRef>>(
        &self,
        input: I,
    ) -> impl Iterator<Item = (MineRef, &MineLocation)> + use<'_, I> {
        input
            .into_iter()
            .map(|mine_ref| (mine_ref, mine_ref.resolve_mine_surface(*self)))
    }

    //

    pub fn all_mines_iter(&self) -> impl Iterator<Item = &MineLocation> {
        self.mines.iter()
    }

    pub fn all_mines_refs_iter(&self) -> impl Iterator<Item = MineRef> + use<> {
        (0..self.mines.len()).map(MineRef)
    }
}

//

pub trait AsVsMut: AsVs {
    fn mines_mut_fn<R>(&mut self, work: impl FnOnce(PlugMut<'_>) -> R) -> R;

    fn mines_mut_old_fn<R>(&mut self, work: impl FnOnce(&mut PlugMut<'_>) -> R) -> R {
        self.mines_mut_fn(|mut s| work(&mut s))
    }

    fn mines_mut_ref(&mut self) -> PlugMut<'_>;
}

pub trait AsVs {
    fn mines(&self) -> Plug<'_>;
}

//

#[derive(PartialEq, Debug, Eq, Hash, Clone, Copy, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(transparent)]
pub struct MineRef(pub(super) usize);

impl MineRef {
    pub fn resolve_mine_surface<'s>(&self, surface: Plug<'s>) -> &'s MineLocation {
        &surface.mines[self.0]
    }

    pub fn resolve_mine_lookup<'plan_mine>(
        &self,
        lookup: &[(MineRef, &'plan_mine MineLocation)],
    ) -> &'plan_mine MineLocation {
        lookup
            .iter()
            .find(|(resolver_ref, _)| resolver_ref == self)
            .unwrap()
            .1
    }
}

// make visible for MineLocation to create
#[derive(PartialEq, Debug, Eq, Hash, Clone, Copy, PartialOrd, Ord, Serialize, Deserialize)]
pub struct MineDestinationRef(pub(in super::super) MineRef, pub(in super::super) usize);

impl MineDestinationRef {
    pub fn resolve_destination_surface<'s>(
        &self,
        surface: VSurfaceMine<'s>,
    ) -> &'s MineDestination {
        &surface.mines[self.0.0].destinations()[self.1]
    }

    pub fn resolve_destination_lookup<'plan_mine>(
        &self,
        lookup: &[(MineRef, &'plan_mine MineLocation)],
    ) -> &'plan_mine MineDestination {
        let mine = lookup
            .iter()
            .find(|(lookup_ref, _)| *lookup_ref == self.0)
            .unwrap()
            .1;
        &mine.destinations()[self.1]
    }

    pub fn mine_ref(&self) -> MineRef {
        self.0
    }
}
