use crate::admiral::lua_command::LuaCommand;
use crate::admiral::trimmer::string_space_shrinker;

#[derive(Debug)]
pub struct RawLuaCommand {
    lua: String,
}

impl RawLuaCommand {
    pub fn new(lua: String) -> Self {
        RawLuaCommand {
            lua: string_space_shrinker(lua),
        }
    }
}

impl LuaCommand for RawLuaCommand {
    fn make_lua(&self) -> String {
        self.lua.clone()
    }
}
