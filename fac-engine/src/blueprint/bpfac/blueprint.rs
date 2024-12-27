use serde::{Deserialize, Serialize};

use crate::blueprint::contents::BlueprintContents;

use super::{entity::FacBpEntity, icons::FacBpIcon, schedule::FacBpSchedule};

#[derive(Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct FacBpBlueprint {
    pub icons: Vec<FacBpIcon>,
    pub entities: Vec<FacBpEntity>,
    pub item: FacBpBlueprintItem,
    pub version: usize,
    #[serde(default)]
    pub schedules: Vec<FacBpSchedule>,
}

#[derive(Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct FacBpBlueprintWrapper {
    pub blueprint: FacBpBlueprint,
}

impl From<BlueprintContents> for FacBpBlueprintWrapper {
    fn from(value: BlueprintContents) -> Self {
        let (_items, entities) = value.consume();
        Self {
            blueprint: FacBpBlueprint {
                entities,
                icons: Vec::new(),
                item: FacBpBlueprintItem::Blueprint,
                version: 5,
                schedules: Vec::new(),
            },
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq)]
pub enum FacBpBlueprintItem {
    Blueprint,
    // todo
    // BlurprintBook,
}
