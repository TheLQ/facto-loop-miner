use std::collections::HashMap;
use std::fmt::Debug;

pub mod fac_destroy;
pub mod fac_execution_define;
pub mod fac_execution_run;
pub mod fac_log;
pub mod fac_surface_create_entity;
pub mod fac_surface_create_entity_safe;
pub mod lua_batch;

pub const DEFAULT_SURFACE_VAR: &str = "game.surfaces[1]";
pub const DEFAULT_FORCE_VAR: &str = "game.forces[1]";

pub fn direction_params(direction: &str) -> HashMap<String, String> {
    direction_params_exact(&format!("defines.direction.{}", direction))
}

pub fn direction_params_exact(value: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    map.insert("direction".to_string(), value.to_string());
    map
}

pub fn recipe_params_exact(value: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    map.insert("recipe".to_string(), format!("\"{}\"", value));
    map
}

pub fn recipe_module_params_exact(value: &str, module: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    map.insert("recipe".to_string(), format!("\"{}\"", value));
    map.insert("modules".to_string(), format!("\"{}\"", module));
    map
}

/// Main Generator - Nestable commands
pub trait LuaCommand: Debug {
    fn make_lua(&self) -> String;
}

pub trait LuaCommandBatch {
    fn make_lua_batch(self, lua_commands: &mut Vec<Box<dyn LuaCommand>>);
}
