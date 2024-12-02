use crate::admiral::lua_command::LuaCommand;

#[derive(Debug)]
pub struct FacExectionRun {}

impl LuaCommand for FacExectionRun {
    fn make_lua(&self) -> String {
        "megacall()".to_string()
    }
}
