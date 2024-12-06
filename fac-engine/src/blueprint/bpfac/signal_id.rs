use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BpFacSignalId {
    name: String,
    #[serde(rename = "type")]
    stype: BpFacSignalIdType,
}

#[derive(Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum BpFacSignalIdType {
    Item,
    Fluid,
    Virtual,
    Entity,
    Recipe,
    SpaceLocation,
    AsteroidChunk,
    Quality,
}
