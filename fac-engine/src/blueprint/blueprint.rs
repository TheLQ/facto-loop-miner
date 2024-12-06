use super::{
    bpfac::blueprint::{BpFacBlueprint, BpFacBlueprintWrapper},
    bpitem::BlueprintItem,
    contents::BlueprintContents,
};

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

    pub fn to_fac(&self) -> BpFacBlueprintWrapper {
        BpFacBlueprintWrapper {
            blueprint: BpFacBlueprint {
                icons: Vec::new(),
                entities: self.contents.to_fac(),
                ..todo!()
            },
        }
    }
}
