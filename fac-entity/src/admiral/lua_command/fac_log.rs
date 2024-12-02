use crate::admiral::lua_command::LuaCommand;

#[derive(Debug)]
pub struct FacLog {
    pub message: String,
}

impl FacLog {
    pub fn new(message: String) -> Self {
        FacLog { message }
    }
}

impl LuaCommand for FacLog {
    fn make_lua(&self) -> String {
        format!("log('{}')", self.message)
    }
}
