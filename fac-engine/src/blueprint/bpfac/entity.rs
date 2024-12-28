use serde::{Deserialize, Serialize};

use crate::{
    admiral::lua_command::fac_surface_create_entity::{CreateParam, FacSurfaceCreateEntity},
    game_entities::{
        belt_split::{FacEntBeltSplitPriority, FacExtPriority},
        belt_under::FacEntBeltUnderType,
        direction::FacDirectionEighth,
        module::FacModule,
    },
};

use super::{
    FacBpInteger, infinity::FacBpInfinitySettings, position::FacBpPosition, schedule::FacBpSchedule,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub station: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub infinity_settings: Option<FacBpInfinitySettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schedule: Option<FacBpSchedule>,
    pub output_priority: FacExtPriority,
    pub input_priority: FacExtPriority,
}

impl FacBpEntity {
    pub fn to_lua(&self) -> FacSurfaceCreateEntity {
        let mut create = FacSurfaceCreateEntity::new(&self.name, self.position.clone());

        if let Some(v) = &self.direction {
            create.with_param(CreateParam::DirectionFacto(v.clone()));
        }
        if let Some(v) = &self.recipe {
            create.with_param(CreateParam::Lua {
                name: "recipe",
                lua: v.into(),
            });
        }
        if let Some(v) = &self.utype {
            create.with_param(CreateParam::Lua {
                name: "type",
                lua: v.to_fac(),
            });
        }
        if let Some(v) = &self.station {
            create.with_param(CreateParam::Lua {
                name: "station",
                lua: v.into(),
            });
        }
        // TODO
        if let Some(v) = &self.items {
            for module in v {
                create.with_command_module(module);
            }
        }
        if let Some(v) = &self.infinity_settings {
            create.with_command_infinity_settings(v);
        }

        if let Some(v) = &self.schedule {
            create.with_command_schedule(v);
        }

        if self.input_priority != FacExtPriority::None
            || self.output_priority != FacExtPriority::None
        {
            create.with_command_splitter(FacEntBeltSplitPriority {
                input: self.input_priority,
                output: self.output_priority,
            });
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
