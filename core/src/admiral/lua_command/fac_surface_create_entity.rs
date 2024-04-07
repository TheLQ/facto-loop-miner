use crate::admiral::lua_command::{LuaCommand, DEFAULT_FORCE_VAR};
use crate::navigator::mori::RailDirection;
use itertools::Itertools;
use opencv::core::Point2f;
use strum::AsRefStr;

const DEBUG_PRE_COLLISION: bool = false;
const DEBUG_POSITION_EXPECTED: bool = true;

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
                CreateParam::Direction(direction) => Some(direction.as_ref()),
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
                .replace('\n', ""),
            )
        }

        if self.commands.is_empty() || DEBUG_POSITION_EXPECTED {
            lua.push("local admiral_create =".to_string());
        }

        lua.push(format!(
            "game.surfaces[1].create_entity{{ \
            name=\"{name}\", \
            position={{ {x}, {y} }}, \
            force={DEFAULT_FORCE_VAR},\
            {params_str}\
            }} ",
        ));

        lua.extend_from_slice(&self.commands);

        if DEBUG_POSITION_EXPECTED {
            lua.push(format!(r#"if admiral_create.position.x ~= {x} or admiral_create.position.y ~= {y} then
            rcon.print("[Admiral] Inserted {name} at {x}x{y} but was placed at " .. admiral_create.position.x .. "x" .. admiral_create.position.y)
            end"#).trim().replace('\n', ""))
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

    pub fn new_rail_straight_facto(position: Point2f, direction: FactoDirection) -> Self {
        Self::new_params(
            "straight-rail",
            position,
            vec![CreateParam::DirectionFacto(direction)],
        )
    }

    pub fn new_rail_curved(position: Point2f, direction: RailDirection) -> Self {
        Self::new_params("curved-rail", position, CreateParam::direction(direction))
    }

    pub fn new_rail_curved_facto(position: Point2f, direction: FactoDirection) -> Self {
        Self::new_params(
            "curved-rail",
            position,
            vec![CreateParam::DirectionFacto(direction)],
        )
    }

    pub fn new_rail_signal(position: Point2f, direction: RailDirection) -> Self {
        Self::new_params(
            "rail-signal",
            position,
            vec![CreateParam::Direction(direction)],
        )
    }

    pub fn new_electric_pole_big(position: Point2f) -> Self {
        Self::new("big-electric-pole", position)
    }

    pub fn new_radar(position: Point2f) -> Self {
        Self::new("radar", position)
    }
}

#[derive(Debug)]
pub enum CreateParam {
    Direction(RailDirection),
    DirectionFacto(FactoDirection),
    Recipe(String),
}

impl CreateParam {
    pub fn to_param(&self) -> (&'static str, String) {
        match self {
            CreateParam::Direction(direction) => (
                "direction",
                format!("defines.direction.{}", direction.as_ref().to_lowercase()),
            ),
            CreateParam::DirectionFacto(direction) => (
                "direction",
                format!("defines.direction.{}", direction.as_ref().to_lowercase()),
            ),
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

#[derive(Debug, AsRefStr)]
pub enum FactoDirection {
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
}
