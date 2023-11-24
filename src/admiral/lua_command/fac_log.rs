use crate::admiral::lua_command::LuaCommand;

#[derive(Debug)]
pub struct FacLog {
    pub message: String,
}

impl LuaCommand for FacLog {
    fn make_lua(&self) -> String {
        format!("log('{}')", self.message)
    }
}
