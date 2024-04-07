use crate::admiral::lua_command::{LuaCommand, DEFAULT_FORCE_VAR};
use crate::navigator::mori::RailDirection;
use itertools::Itertools;
use opencv::core::Point2f;
use strum::AsRefStr;

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
        let create = format!(
            "game.surfaces[1].create_entity{{ \
            name=\"{}\", \
            position={{ x={},y={} }}, \
            force={},\
            {}\
            }};",
            self.name, self.position.x, self.position.y, DEFAULT_FORCE_VAR, params_str
        );

        if self.commands.is_empty() {
            create
        } else {
            format!(
                "local admiral_create = {} {}",
                create,
                self.commands.join(" ")
            )
        }
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
