use crate::admiral::lua_command::raw_lua::RawLuaCommand;

const RAW_SCAN: &str = include_str!("../../../../scanner_mod/scanner.lua");

pub fn facscan_hyper_scan() -> Vec<RawLuaCommand> {
    extract_commands_from_scanner_mod("hyper_scan()")
}

pub fn facscan_mega_export_entities_compressed() -> Vec<RawLuaCommand> {
    extract_commands_from_scanner_mod("mega_export_entities_compressed()")
}

fn extract_commands_from_scanner_mod(lua_function: &str) -> Vec<RawLuaCommand> {
    // grab the define
    let start_str = format!("function {}", lua_function);

    // local function ->hyper_scan()<-
    let Some(start_pos) = RAW_SCAN.find(&start_str) else {
        panic!("{} not found in scanner", start_str)
    };

    // function is called immediately after definition
    let end_search_pos = start_pos + lua_function.len();
    let Some(end_pos) = RAW_SCAN[end_search_pos..].find(lua_function) else {
        panic!("{} not found in scanner", lua_function)
    };
    let end_pos = end_search_pos + end_pos;

    let extracted_command = &RAW_SCAN[start_pos..end_pos].trim();

    let mut res = Vec::new();
    res.push(RawLuaCommand::new(extracted_command.to_string()));
    res.push(RawLuaCommand::new(lua_function.to_string()));
    res
}
