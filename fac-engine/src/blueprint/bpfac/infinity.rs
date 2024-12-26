use serde::{Deserialize, Serialize};

use super::FacBpInteger;

#[derive(Debug, PartialEq, Clone, Default, Serialize, Deserialize)]
pub struct FacBpInfinitySettings {
    pub remove_unfiltered_items: bool,
    pub filters: Vec<FacBpFilter>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct FacBpFilter {
    pub name: String,
    pub count: FacBpInteger,
    pub mode: String,
}
