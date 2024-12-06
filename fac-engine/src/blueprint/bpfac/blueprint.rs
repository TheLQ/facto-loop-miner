use serde::{Deserialize, Serialize};

use super::{entity::BpFacEntity, icons::BpFacIcon};

#[derive(Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BpFacBlueprint {
    pub icons: Vec<BpFacIcon>,
    pub entities: Vec<BpFacEntity>,
    pub item: String,
    pub version: usize,
}

#[derive(Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BpFacBlueprintWrapper {
    pub blueprint: BpFacBlueprint,
}
