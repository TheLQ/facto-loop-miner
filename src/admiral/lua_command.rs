use crate::surface::surface::PointU32;
use itertools::Itertools;
use opencv::core::{Point, Point2f};
use std::collections::HashMap;
use std::fmt::format;

pub const DEFAULT_SURFACE_VAR: &str = "game.surfaces[1]";
pub const DEFAULT_FORCE_VAR: &str = "game.forces[1]";

pub trait LuaCommand {
    fn make_lua(&self) -> String;
}

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

pub struct FacLog {
    pub message: String,
}

impl LuaCommand for FacLog {
    fn make_lua(&self) -> String {
        format!("log('{}')", self.message)
    }
}

pub struct FacDestroy {}

impl LuaCommand for FacDestroy {
    fn make_lua(&self) -> String {
        // for entity in
        format!(
            r#"
local entities = game.surfaces[1].find_entities({{ {{ 0,0 }} , {{ 1000, 1000 }} }})
for _, entity in ipairs(entities) do
    log('destroying ' .. tostring(entity.object_name) )
    entity.destroy()
end
rcon.print('destroy_success')
        "#
        )
    }
}
