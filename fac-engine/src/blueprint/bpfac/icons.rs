use super::{BpFacInteger, signal_id::BpFacSignalId};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct FacBpIcon {
    pub index: BpFacInteger,
    pub signal: BpFacSignalId,
}
