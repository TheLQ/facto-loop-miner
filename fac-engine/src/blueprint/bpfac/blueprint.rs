use serde::{Deserialize, Serialize};

use super::{entity::FacBpEntity, icons::FacBpIcon, schedule::FacBpSchedule};

#[derive(Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct FacBpBlueprint {
    pub icons: Vec<FacBpIcon>,
    pub entities: Vec<FacBpEntity>,
    pub item: String,
    pub version: usize,
    #[serde(default)]
    pub schedules: Vec<FacBpSchedule>,
}

#[derive(Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct FacBpBlueprintWrapper {
    pub blueprint: FacBpBlueprint,
}
