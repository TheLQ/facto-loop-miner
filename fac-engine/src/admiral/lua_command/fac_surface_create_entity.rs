use crate::admiral::lua_command::{DEFAULT_FORCE_VAR, LuaCommand};
use crate::common::cvpoint::Point2f;
use crate::common::vpoint::VPoint;
use crate::game_entities::direction::FacDirectionEighth;
use crate::game_entities::direction::RailDirection;
use itertools::Itertools;
use std::convert::AsRef;

pub const DEBUG_PRE_COLLISION: bool = false;
pub const DEBUG_POSITION_EXPECTED: bool = false;

#[derive(Debug)]
pub struct FacSurfaceCreateEntity {
    pub name: String,
    pub position: Point2f,
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
    pub fn new(name: &'static str, position: Point2f) -> Self {
        FacSurfaceCreateEntity {
            name: name.to_string(),
            position,
            params: Vec::new(),
            commands: Vec::new(),
        }
    }

    pub fn new_params(name: &'static str, position: Point2f, params: Vec<CreateParam>) -> Self {
        FacSurfaceCreateEntity {
            name: name.to_string(),
            position,
            params,
            commands: Vec::new(),
        }
    }

    pub fn new_commands(name: &'static str, position: Point2f, commands: Vec<String>) -> Self {
        FacSurfaceCreateEntity {
            name: name.to_string(),
            position,
            params: Vec::new(),
            commands,
        }
    }

    pub fn new_params_commands(
        name: &'static str,
        position: Point2f,
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

    pub fn new_rail_straight(position: Point2f, direction: RailDirection) -> Self {
        Self::new_params("straight-rail", position, CreateParam::direction(direction))
    }

    pub fn new_rail_straight_facto(position: Point2f, direction: FacDirectionEighth) -> Self {
        Self::new_params("straight-rail", position, vec![
            CreateParam::DirectionFacto(direction),
        ])
    }

    pub fn new_rail_curved(position: Point2f, direction: RailDirection) -> Self {
        Self::new_params("curved-rail", position, CreateParam::direction(direction))
    }

    pub fn new_rail_curved_facto(position: Point2f, direction: FacDirectionEighth) -> Self {
        Self::new_params("curved-rail", position, vec![CreateParam::DirectionFacto(
            direction,
        )])
    }

    pub fn new_rail_signal(position: Point2f, direction: RailDirection) -> Self {
        Self::new_params("rail-signal", position, vec![CreateParam::Direction(
            direction,
        )])
    }

    pub fn new_electric_pole_big(position: Point2f) -> Self {
        Self::new("big-electric-pole", position)
    }

    pub fn new_radar(position: Point2f) -> Self {
        Self::new("radar", position)
    }

    pub fn new_drill(position: Point2f, direction: RailDirection) -> Self {
        Self::new_params("electric-mining-drill", position, vec![
            CreateParam::Direction(direction),
        ])
    }

    pub fn new_chest_red(position: Point2f) -> Self {
        Self::new("logistic-chest-passive-provider", position)
    }

    pub fn new_electric_pole_medium(position: VPoint) -> Self {
        Self::new("medium-electric-pole", position.to_f32_with_offset(0.5))
    }

    pub fn new_substation(position: VPoint) -> Self {
        Self::new("substation", position.to_f32_with_offset(1.0))
    }

    pub fn new_roboport(position: VPoint) -> Self {
        Self::new("roboport", position.to_f32_with_offset(1.5))
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
