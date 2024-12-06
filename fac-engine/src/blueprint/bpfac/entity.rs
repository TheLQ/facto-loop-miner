use serde::{Deserialize, Serialize};

use crate::game_entities::direction::FacDirectionEighth;

use super::{BpFacInteger, position::BpFacPosition};

#[derive(Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub struct BpFacEntity {
    #[serde(rename = "entity_number")]
    pub entity_number: BpFacInteger,
    pub name: String,
    pub position: BpFacPosition,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<FacDirectionEighth>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub neighbours: Option<Vec<BpFacInteger>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recipe: Option<String>,
}

#[cfg(test)]
mod test {
    use super::BpFacEntity;

    #[test]
    fn basic() {
        let raw = r#"{"entity_number":1,"name":"logistic-chest-passive-provider","position":{"x":-6.5,"y":-4.5}}"#;
        let _: BpFacEntity = serde_json::from_str(raw).unwrap();
    }
}
