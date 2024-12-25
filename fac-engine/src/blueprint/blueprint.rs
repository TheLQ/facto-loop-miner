use super::contents::BlueprintContents;

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
}
