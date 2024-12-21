use serde::{Deserialize, Serialize};

use crate::{
    admiral::lua_command::fac_surface_create_entity::{CreateParam, FacSurfaceCreateEntity},
    game_entities::{
        belt_under::FacEntBeltUnderType, direction::FacDirectionEighth, module::FacModule,
    },
};

use super::{FacBpInteger, position::FacBpPosition};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub struct FacBpEntity {
    #[serde(rename = "entity_number")]
    pub entity_number: FacBpInteger,
    pub name: String,
    pub position: FacBpPosition,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<FacDirectionEighth>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub neighbours: Option<Vec<FacBpInteger>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recipe: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Vec<FacModule>>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "type")]
    pub utype: Option<FacEntBeltUnderType>,
}

impl FacBpEntity {
    pub fn to_lua(&self) -> FacSurfaceCreateEntity {
        let mut create = FacSurfaceCreateEntity::new(&self.name, self.position.clone());

        if let Some(v) = &self.direction {
            create.with_param(CreateParam::DirectionFacto(v.clone()));
        }
        if let Some(v) = &self.recipe {
            create.with_param(CreateParam::Recipe(v.clone()));
        }
        if let Some(v) = &self.utype {
            create.with_param(CreateParam::Type(v.to_fac()));
        }
        // TODO
        if let Some(v) = &self.items {
            for module in v {
                create.with_command_module(module);
            }
        }

        create
    }
}

#[cfg(test)]
mod test {
    use super::FacBpEntity;

    #[test]
    fn basic() {
        let raw = r#"{"entity_number":1,"name":"logistic-chest-passive-provider","position":{"x":-6.5,"y":-4.5}}"#;
        let _: FacBpEntity = serde_json::from_str(raw).unwrap();
    }
}
