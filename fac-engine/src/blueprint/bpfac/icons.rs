use super::{FacBpInteger, signal_id::FacBpSignalId};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct FacBpIcon {
    pub index: FacBpInteger,
    pub signal: FacBpSignalId,
}
