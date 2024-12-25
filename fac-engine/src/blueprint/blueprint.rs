use super::{bpitem::BlueprintItem, contents::BlueprintContents};

pub struct Blueprint {
    contents: BlueprintContents,
}

impl Blueprint {
    pub fn new(contents: BlueprintContents) -> Self {
        Self { contents }
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

    #[cfg(test)] // no log context
    pub fn to_fac(&self) -> super::bpfac::blueprint::FacBpBlueprintWrapper {
        use super::bpfac::blueprint::{FacBpBlueprint, FacBpBlueprintWrapper};
        FacBpBlueprintWrapper {
            blueprint: FacBpBlueprint {
                icons: Vec::new(),
                entities: self.contents.to_fac(),
                item: "blueprint".into(),
                version: 0,
            },
        }
    }
}
