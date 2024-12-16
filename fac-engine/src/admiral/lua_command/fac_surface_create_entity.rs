use crate::admiral::lua_command::{DEFAULT_FORCE_VAR, LuaCommand};
use crate::blueprint::bpfac::position::FacBpPosition;
use crate::game_entities::direction::FacDirectionEighth;
use crate::game_entities::direction::RailDirection;
use itertools::Itertools;
use std::convert::AsRef;

pub const DEBUG_PRE_COLLISION: bool = false;
pub const DEBUG_POSITION_EXPECTED: bool = false;

#[derive(Debug)]
pub struct FacSurfaceCreateEntity {
    pub name: String,
    pub position: FacBpPosition,
    pub params: Vec<CreateParam>,
    pub commands: Vec<String>,
}

impl LuaCommand for FacSurfaceCreateEntity {
    fn make_lua(&self) -> String {
        let params_str = self
            .params
            .iter()
            .map(|v| {
                let (key, value) = v.to_param();
                format!("{}={}", key, value)
            })
            .join(",");

        let mut lua: Vec<String> = Vec::new();

        let name = &self.name;
        let x = self.position.x;
        let y = self.position.y;

        if DEBUG_PRE_COLLISION {
            let direction = self.params.iter().find_map(|v| match v {
                CreateParam::Direction(direction) => Some(direction.to_factorio()),
                CreateParam::DirectionFacto(direction) => Some(direction.as_ref()),
                _ => None,
            });

            let direction_param = if let Some(direction) = direction {
                format!("defines.direction.{}", direction.to_lowercase())
            } else {
                "".to_string()
            };

            lua.push(
                format!(
                    r#"
                    if game.surfaces[1].entity_prototype_collides("{name}", {{ {x}, {y} }}, false, {direction_param}) then
                        rcon.print("[Admiral] Collision {name} {x}x{y}")           
                    end 
                    "#
                )
                .trim()
                .replace('\n', "")
                .replace("    ", ""),
            )
        }

        if !self.commands.is_empty() || DEBUG_POSITION_EXPECTED {
            lua.push("local admiral_create =".to_string());
        }

        lua.push(
            format!(
                r#"game.surfaces[1].create_entity{{ 
                    name="{name}", 
                    position={{ {x}, {y} }}, 
                    force={DEFAULT_FORCE_VAR},
                    {params_str}
                }}"#,
            )
            .trim()
            .replace('\n', "")
            .replace("    ", ""),
        );

        lua.extend_from_slice(&self.commands);

        if DEBUG_POSITION_EXPECTED {
            lua.push(format!(
                r#"if admiral_create == nil then
                    rcon.print("[Admiral] Inserted {name} at {x}x{y} but was nil")
                elseif admiral_create.position.x ~= {x} or admiral_create.position.y ~= {y} then
                    rcon.print("[Admiral] Inserted {name} at {x}x{y} but was placed at " .. admiral_create.position.x .. "x" .. admiral_create.position.y)
                end"#
            ).trim().replace('\n', ""));
        }

        lua.join(" ")
    }
}

impl FacSurfaceCreateEntity {
    pub fn new(name: &str, position: FacBpPosition) -> Self {
        FacSurfaceCreateEntity {
            name: name.to_string(),
            position,
            params: Vec::new(),
            commands: Vec::new(),
        }
    }

    pub fn new_params(name: &str, position: FacBpPosition, params: Vec<CreateParam>) -> Self {
        FacSurfaceCreateEntity {
            name: name.to_string(),
            position,
            params,
            commands: Vec::new(),
        }
    }

    pub fn new_commands(name: &str, position: FacBpPosition, commands: Vec<String>) -> Self {
        FacSurfaceCreateEntity {
            name: name.to_string(),
            position,
            params: Vec::new(),
            commands,
        }
    }

    pub fn new_params_commands(
        name: &str,
        position: FacBpPosition,
        params: Vec<CreateParam>,
        commands: Vec<String>,
    ) -> Self {
        FacSurfaceCreateEntity {
            name: name.to_string(),
            position,
            params,
            commands,
        }
    }

    pub fn with_param(&mut self, param: CreateParam) {
        self.params.push(param);
    }
}

#[derive(Debug)]
pub enum CreateParam {
    Direction(RailDirection),
    DirectionFacto(FacDirectionEighth),
    Recipe(String),
}

impl CreateParam {
    pub fn to_param(&self) -> (&'static str, String) {
        match self {
            CreateParam::Direction(direction) => (
                "direction",
                format!("defines.direction.{}", direction.to_factorio()),
            ),
            CreateParam::DirectionFacto(direction) => {
                let direction: &str = direction.as_ref();
                (
                    "direction",
                    format!("defines.direction.{}", direction.to_lowercase()),
                )
            }
            CreateParam::Recipe(name) => ("recipe", name.clone()),
        }
    }

    pub fn direction(direction: RailDirection) -> Vec<Self> {
        vec![CreateParam::Direction(direction)]
    }

    pub fn recipe_str(recipe: &'static str) -> Vec<Self> {
        vec![CreateParam::Recipe(recipe.to_string())]
    }
}
