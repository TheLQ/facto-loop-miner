use crate::admiral::generators::join_commands;
use crate::surface::surface::PointU32;
use itertools::Itertools;
use opencv::core::{Point, Point2f};
use std::collections::HashMap;
use std::fmt::{format, Debug, Formatter};

pub const DEFAULT_SURFACE_VAR: &str = "game.surfaces[1]";
pub const DEFAULT_FORCE_VAR: &str = "game.forces[1]";

pub fn direction_params(direction: &str) -> HashMap<String, String> {
    direction_params_exact(&format!("defines.direction.{}", direction))
}

pub fn direction_params_exact(direction: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    map.insert("direction".to_string(), direction.to_string());
    map
}

/// Main Generator - Nestable commands
pub trait LuaCommand: Debug {
    fn make_lua(&self) -> String;
}

pub trait LuaCommandBatch {
    fn make_lua_batch(self) -> Vec<Box<dyn LuaCommand>>;
}

#[derive(Debug)]
pub struct FacSurfaceCreateEntity {
    pub surface_var: String,
    pub name: String,
    pub position: Point2f,
    pub params: HashMap<String, String>,
}

impl LuaCommand for FacSurfaceCreateEntity {
    fn make_lua(&self) -> String {
        let params_str = self
            .params
            .iter()
            .map(|(key, value)| format!("{}={}", key, value))
            .join(",");
        format!(
            "{}.create_entity{{ \
            name=\"{}\", \
            position={{ x={},y={} }}, \
            force={},\
            {}\
            }};",
            self.surface_var,
            self.name,
            self.position.x,
            self.position.y,
            DEFAULT_FORCE_VAR,
            params_str
        )
    }
}

#[derive(Debug)]
pub struct FacSurfaceCreateEntitySafe {
    pub inner: FacSurfaceCreateEntity,
}

impl LuaCommand for FacSurfaceCreateEntitySafe {
    fn make_lua(&self) -> String {
        format!(
            r#"function FacSurfaceCreateEntitySafe()
local admiral_create = {}
if admiral_create == nil then
    rcon.print('create_entity_failed')
elseif admiral_create.position.x ~= {} or admiral_create.position.y ~= {} then
    rcon.print('create_entity_bad_position')
    log("created at {1}x{2} placed at " .. admiral_create.position.x .. "x" .. admiral_create.position.y .. "y")
    print("created at {1}x{2} placed at " .. admiral_create.position.x .. "x" .. admiral_create.position.y .. "y")
else
    rcon.print('create_entity_success')
end
end
FacSurfaceCreateEntitySafe()"#,
            self.inner.make_lua(),
            self.inner.position.x,
            self.inner.position.y,
        )
    }
}

#[derive(Debug)]
pub struct FacLog {
    pub message: String,
}

impl LuaCommand for FacLog {
    fn make_lua(&self) -> String {
        format!("log('{}')", self.message)
    }
}

#[derive(Debug)]
pub struct FacDestroy {}

impl LuaCommand for FacDestroy {
    fn make_lua(&self) -> String {
        // for entity in
        format!(
            r#"
local entities = game.surfaces[1].find_entities({{ {{ 0,0 }} , {{ 1000, 1000 }} }})
for _, entity in ipairs(entities) do
    rcon.print('destroy_' .. entity.name )
    entity.destroy()
end
rcon.print('destroy_success')
        "#
        )
    }
}

#[derive(Debug)]
pub struct FacExectionDefine {
    pub commands: Vec<Box<dyn LuaCommand>>,
}

impl LuaCommand for FacExectionDefine {
    fn make_lua(&self) -> String {
        let all_function_chunks = self
            .commands
            .iter()
            .chunks(100)
            .into_iter()
            .enumerate()
            .map(|(i, v)| {
                let mut inner_function = format!("local chunk = {} function megachunk()\n", i);
                inner_function.push_str(&join_commands(self.commands.iter()));
                inner_function.push_str("\nend");
                inner_function.push_str("\nmegachunk()\n");
                inner_function
            })
            .join("\n");
        format!(
            r#"
function megacall()
{}
end
        "#,
            all_function_chunks
        )
    }
}

#[derive(Debug)]
pub struct FacExectionRun {}

impl LuaCommand for FacExectionRun {
    fn make_lua(&self) -> String {
        "megacall()".to_string()
    }
}

#[derive(Debug)]
pub struct BasicLuaBatch {
    pub commands: Vec<Box<dyn LuaCommand>>,
}

impl LuaCommandBatch for BasicLuaBatch {
    fn make_lua_batch(self) -> Vec<Box<dyn LuaCommand>> {
        self.commands
    }
}
