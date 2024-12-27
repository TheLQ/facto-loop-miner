use super::{bpfac::entity::FacBpEntity, bpitem::BlueprintItem};

pub struct BlueprintContents {
    items: Vec<BlueprintItem>,
    fac_entities: Vec<FacBpEntity>,
}

impl BlueprintContents {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
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

    pub fn consume(self) -> (Vec<BlueprintItem>, Vec<FacBpEntity>) {
        (self.items, self.fac_entities)
    }
}
