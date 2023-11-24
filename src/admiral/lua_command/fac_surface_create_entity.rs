use crate::admiral::lua_command::{LuaCommand, DEFAULT_FORCE_VAR};
use itertools::Itertools;
use opencv::core::Point2f;
use std::collections::HashMap;

#[derive(Debug)]
pub struct FacSurfaceCreateEntity {
    pub surface_var: String,
    pub name: String,
    pub position: Point2f,
    pub params: HashMap<String, String>,
    pub extra: Vec<String>,
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
