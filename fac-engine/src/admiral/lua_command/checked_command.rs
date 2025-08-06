use crate::admiral::lua_command::LuaCommand;
use rand::{RngCore, rng};

#[derive(Debug)]
pub struct CheckedLuaCommand {
    id: u32,
    inner: Box<dyn LuaCommand>,
}

impl CheckedLuaCommand {
    pub fn new(inner: Box<dyn LuaCommand>) -> Self {
        CheckedLuaCommand {
            id: rng().next_u32(),
            inner,
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }
}

impl LuaCommand for CheckedLuaCommand {
    fn make_lua(&self) -> String {
        format!("{} rcon.print('{}')", self.inner.make_lua(), self.id)
    }
}
