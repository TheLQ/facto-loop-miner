use crate::admiral::lua_command::fac_surface_create_entity::FacSurfaceCreateEntity;
use crate::admiral::lua_command::LuaCommand;

#[derive(Debug)]
struct FacSurfaceCreateEntitySafe {
    pub inner: FacSurfaceCreateEntity,
}

impl LuaCommand for FacSurfaceCreateEntitySafe {
    fn make_lua(&self) -> String {
        let lua_text = self.inner.make_lua();
        format!(
            r#"
admiral_create = {}
{}
if admiral_create == nil then
    rcon.print('create_entity_failed')
elseif admiral_create.position.x ~= {} or admiral_create.position.y ~= {} then
    rcon.print('create_entity_bad_position')
    rcon.print("created at {2}x{3} placed at " .. admiral_create.position.x .. "x" .. admiral_create.position.y .. "y entity {}")
else
    rcon.print('create_entity_success_{}')
end"#,
            self.inner.make_lua(),
            // self.inner.extra.join("\n"),
            "",
            self.inner.position.x,
            self.inner.position.y,
            format!("{:?}", self.inner)
                .replace("\\\"", "")
                .replace('\"', ""),
            self.inner.name
        )
    }
}
