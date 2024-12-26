use serde::{Deserialize, Serialize};

use super::FacBpInteger;

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct FacBpSchedule {
    pub locomotives: Vec<FacBpInteger>,
    pub schdata: Vec<FacBpScheduleData>,
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct FacBpScheduleData {
    pub station: String,
    pub wait_conditions: Vec<FacBpScheduleWait>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FacBpScheduleWait {
    pub compare_type: FacBpLogic,
    #[serde(rename = "type")]
    pub ctype: FacBpWaitType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<FacBpCircuitCondition>,
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct FacBpCircuitCondition {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constant: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comparator: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_signal: Option<FacBpSignalId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub second_signal: Option<FacBpSignalId>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FacBpSignalId {
    #[serde(rename = "type")]
    pub stype: FacBpSignalIdType,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FacBpSignalIdType {
    Item,
    Fluid,
    Virual,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FacBpLogic {
    And,
    Or,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FacBpWaitType {
    Time,
    Full,
    Empty,
    ItemCount,
    Circuit,
    Inactivity,
    RobotsInactive,
    FluidCount,
    PassenterPresent,
    PassengerNotPresent,
}
