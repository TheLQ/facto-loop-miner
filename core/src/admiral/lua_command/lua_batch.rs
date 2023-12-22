use crate::admiral::lua_command::{LuaCommand, LuaCommandBatch};

#[derive(Debug)]
pub struct BasicLuaBatch {
    pub commands: Vec<Box<dyn LuaCommand>>,
}

impl LuaCommandBatch for BasicLuaBatch {
    fn make_lua_batch(self, lua_commands: &mut Vec<Box<dyn LuaCommand>>) {
        for command in self.commands {
            lua_commands.push(command);
        }
    }
}
