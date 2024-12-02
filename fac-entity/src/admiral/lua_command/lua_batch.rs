use crate::LOCALE;
use crate::admiral::lua_command::LuaCommand;
use itertools::Itertools;
use num_format::ToFormattedString;
use tracing::trace;

#[derive(Debug)]
pub struct LuaBatchCommand {
    pub commands: Vec<Box<dyn LuaCommand>>,
}

impl LuaBatchCommand {
    pub fn new(commands: Vec<Box<dyn LuaCommand>>) -> Self {
        trace!(
            "execute batch of {}",
            commands.len().to_formatted_string(&LOCALE)
        );
        Self { commands }
    }
}

impl LuaCommand for LuaBatchCommand {
    fn make_lua(&self) -> String {
        self.commands.iter().map(|c| c.make_lua()).join(" ")
    }
}

// impl LuaCommandBatch for BasicLuaBatch {
//     fn make_lua_batch(self, lua_commands: &mut Vec<Box<dyn LuaCommand>>) {
//         for command in self.commands {
//             lua_commands.push(command);
//         }
//     }
// }
