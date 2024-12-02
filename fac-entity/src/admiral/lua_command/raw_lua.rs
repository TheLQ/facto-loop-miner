use crate::admiral::lua_command::LuaCommand;

#[derive(Debug)]
pub struct RawLuaCommand {
    lua: String,
}

impl RawLuaCommand {
    pub fn new(lua: String) -> Self {
        RawLuaCommand { lua }
    }
}

impl LuaCommand for RawLuaCommand {
    fn make_lua(&self) -> String {
        self.lua.clone()
    }
}
