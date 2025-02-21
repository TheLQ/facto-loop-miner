use super::{
    bpfac::{entity::FacBpEntity, tile::FacBpTile},
    bpitem::BlueprintItem,
};
use crate::blueprint::bpfac::blueprint::FacBpBlueprintWrapper;

pub struct BlueprintContents {
    items: Vec<BlueprintItem>,
    fac_tiles: Vec<FacBpTile>,
    fac_entities: Vec<FacBpEntity>,
}

impl BlueprintContents {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            fac_tiles: Vec::new(),
            fac_entities: Vec::new(),
        }
    }

    pub fn items(&self) -> &[BlueprintItem] {
        &self.items
    }

    pub fn fac_entities(&self) -> &[FacBpEntity] {
        &self.fac_entities
    }

    pub fn add(&mut self, item: BlueprintItem, fac_entity: FacBpEntity) {
        self.items.push(item);
        self.fac_entities.push(fac_entity);
    }

    pub fn add_tile(&mut self, fac_tile: FacBpTile) {
        self.fac_tiles.push(fac_tile);
    }

    pub fn consume(self) -> (Vec<BlueprintItem>, Vec<FacBpEntity>) {
        (self.items, self.fac_entities)
    }

    pub fn into_bp(self) -> FacBpBlueprintWrapper {
        self.into()
    }
}
