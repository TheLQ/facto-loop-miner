use super::{bpitem::BlueprintItem, contents::BlueprintContents};

pub struct Blueprint {
    contents: BlueprintContents,
}

impl Blueprint {
    pub fn new() -> Self {
        Self {
            contents: BlueprintContents::new(),
        }
    }

    pub fn contents(&self) -> &BlueprintContents {
        &self.contents
    }

    pub fn contents_mut(&mut self) -> &mut BlueprintContents {
        &mut self.contents
    }

    pub fn inner_entities(&self) -> &[BlueprintItem] {
        self.contents.entities()
    }
}
