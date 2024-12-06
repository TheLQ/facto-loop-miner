use super::bpitem::BlueprintItem;

pub struct BlueprintContents {
    entities: Vec<BlueprintItem>,
}

impl BlueprintContents {
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
        }
    }

    pub fn entities(&self) -> &[BlueprintItem] {
        &self.entities
    }

    pub fn add_entity_each(&mut self, bpitem: BlueprintItem) {
        self.entities.push(bpitem);
    }
}
