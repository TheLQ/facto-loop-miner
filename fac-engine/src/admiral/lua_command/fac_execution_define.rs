use crate::admiral::generators::join_commands;
use crate::admiral::lua_command::LuaCommand;
use itertools::Itertools;

#[derive(Debug)]
pub struct FacExectionDefine {
    pub commands: Vec<Box<dyn LuaCommand>>,
}

impl LuaCommand for FacExectionDefine {
    fn make_lua(&self) -> String {
        let all_function_chunks = self
            .commands
            .iter()
            .chunks(75)
            .into_iter()
            .enumerate()
            .map(|(i, v)| {
                let mut inner_function = format!("chunk = {i} function megachunk()\n");
                inner_function.push_str("\nlocal admiral_create = nil\n");
                inner_function.push_str(&join_commands(v));
                inner_function.push_str("\nend");
                inner_function.push_str("\nmegachunk()\n");
                inner_function
            })
            .join("\n")
            .to_string();

        format!(
            r"
function megacall()
{all_function_chunks}
end rcon.print('facexecution_define')"
        )
    }
}
