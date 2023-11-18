use crate::surface::surface::PointU32;
use opencv::core::{Point, Point2f};
use std::fmt::format;

pub trait LuaCommand {
    fn make_lua(&self) -> String;
}

pub struct FacSurfaceCreateEntity {
    pub(crate) surface_var: String,
    pub(crate) name: String,
    pub(crate) position: Point2f,
}

impl LuaCommand for FacSurfaceCreateEntity {
    fn make_lua(&self) -> String {
        format!(
            "{}.create_entity{{ \
            name=\"{}\", \
            position={{ x={},y={} }} \
            }};",
            self.surface_var, self.name, self.position.x, self.position.y
        )
    }
}

pub struct FacSurfaceCreateEntitySafe {
    pub inner: FacSurfaceCreateEntity,
}

impl LuaCommand for FacSurfaceCreateEntitySafe {
    fn make_lua(&self) -> String {
        format!(
            r#"local admiral_create = {}
if admiral_create == nil then
    rcon.print('create_entity_failed')
else
    rcon.print('create_entity_success')
end"#,
            self.inner.make_lua()
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
