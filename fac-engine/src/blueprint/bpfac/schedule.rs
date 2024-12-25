use serde::{Deserialize, Serialize};

use super::FacBpInteger;

#[derive(PartialEq, Serialize, Deserialize)]
pub struct FacBpSchedule {
    locomotives: Vec<FacBpInteger>,
    schedule: Vec<FacBpScheduleData>,
}

#[derive(PartialEq, Serialize, Deserialize)]
pub struct FacBpScheduleData {
    station: String,
    wait_conditions: Vec<FacBpScheduleCondition>,
}

#[derive(PartialEq, Serialize, Deserialize)]
pub struct FacBpScheduleCondition {
    compare_type: String,
    #[serde(rename = "type")]
    ctype: String,
}
