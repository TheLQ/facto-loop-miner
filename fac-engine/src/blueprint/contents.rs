use super::{bpfac::entity::BpFacEntity, bpitem::BlueprintItem};

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

    pub fn to_fac(&self) -> Vec<BpFacEntity> {
        let mut counter = 0;
        self.entities
            .iter()
            .map(|i| {
                let entity_number = counter;
                counter += 1;
                i.entity().to_fac(entity_number, i.position())
            })
            .collect()
    }
}
