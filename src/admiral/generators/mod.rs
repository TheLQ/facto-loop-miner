use crate::admiral::lua_command::LuaCommand;

pub mod assembler_farm;
pub mod beacon_farm;
pub mod rail90;
pub mod rail_beacon_farm;
pub mod rail_line;
pub mod rail_station;

pub fn join_commands<'a>(
    creation_commands: impl Iterator<Item = &'a Box<dyn LuaCommand>>,
) -> String {
    let mut result = "".to_string();
    for command in creation_commands {
        result.push_str(&command.make_lua());
        result.push('\n');
    }
    result
}
