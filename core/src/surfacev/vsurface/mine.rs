use crate::surfacev::mine::MineLocation;
use serde::{Deserialize, Serialize};
use simd_json::prelude::ArrayTrait;
use std::borrow::Borrow;

pub struct PlugMut<'s> {
    pub(super) mines: &'s mut Vec<MineLocation>,
}

#[derive(Clone, Copy)]
pub struct Plug<'s> {
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
        input: impl IntoIterator<Item = impl Borrow<MineRef>>,
    ) -> impl Iterator<Item = &MineLocation> {
        input.into_iter().map(|v| v.borrow().get_mine(*self))
    }

    pub fn resolve_mines_with_refs(
        &self,
        input: impl IntoIterator<Item = MineRef>,
    ) -> impl Iterator<Item = (MineRef, &MineLocation)> {
        input
            .into_iter()
            .map(|mine_ref| (mine_ref, mine_ref.get_mine(*self)))
    }
}

//

pub trait AsVsMut: AsVs {
    fn mines_mut(&mut self) -> PlugMut<'_>;
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
    pub fn get_mine<'s>(&self, surface: Plug<'s>) -> &'s MineLocation {
        &surface.mines[self.0]
    }
}
