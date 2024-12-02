use crate::admiral::generators::join_commands;
use crate::admiral::lua_command::LuaCommand;
use itertools::Itertools;
use regex::Regex;

#[derive(Debug)]
pub struct FacExectionDefine {
    pub commands: Vec<Box<dyn LuaCommand>>,
}

impl LuaCommand for FacExectionDefine {
    fn make_lua(&self) -> String {
        let mut all_function_chunks = self
            .commands
            .iter()
            .chunks(75)
            .into_iter()
            .enumerate()
            .map(|(i, v)| {
                let mut inner_function = format!("chunk = {} function megachunk()\n", i);
                inner_function.push_str("\nlocal admiral_create = nil\n");
                inner_function.push_str(&join_commands(v));
                inner_function.push_str("\nend");
                inner_function.push_str("\nmegachunk()\n");
                inner_function
            })
            .join("\n")
            .replace('\n', " ")
            .to_string();

        let regex = Regex::new("( \\s+)").unwrap();
        all_function_chunks = regex
            .unwrap()
            .replace_all(&all_function_chunks, " ")
            .to_string();

        format!(
            r#"
function megacall()
{}
end rcon.print('facexecution_define')
        "#,
            all_function_chunks
        )
    }
}
