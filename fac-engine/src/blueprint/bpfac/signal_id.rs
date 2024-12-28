use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FacBpSignalId {
    pub name: String,
    #[serde(rename = "type")]
    pub stype: FacBpSignalIdType,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FacBpSignalIdType {
    Item,
    Fluid,
    Virtual,
    Entity,
    Recipe,
    SpaceLocation,
    AsteroidChunk,
    Quality,
}
