use crate::admiral::lua_command::raw_lua::RawLuaCommand;
use crate::admiral::lua_command::LuaCommand;
use crate::surfacev::vsurface::VSurface;
use crate::TILES_PER_CHUNK;
use itertools::Itertools;

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

#[derive(Debug)]
pub struct BaseScanner {
    radius: u32,
}

impl BaseScanner {
    pub fn new_radius(radius: u32) -> Self {
        BaseScanner { radius }
    }

    pub fn new(surface: &VSurface) -> Self {
        Self::new_radius(surface.get_radius())
    }
}

impl LuaCommand for BaseScanner {
    fn make_lua(&self) -> String {
        // not sure if chunks on the exact edge are generated...
        let chunks = self.radius as usize / TILES_PER_CHUNK;
        format!(r#"
if not game.surfaces[1].is_chunk_generated({{ -{chunks}, -{chunks} }}) or not game.surfaces[1].is_chunk_generated({{ {chunks}, {chunks} }})
then
    log("Generating {chunks} Chunks...")
    game.surfaces[1].request_to_generate_chunks({{ 0, 0 }}, {chunks})
    log("force_generate....")
    game.surfaces[1].force_generate_chunk_requests()
    log("Generate Complete")
end
        "#).trim().replace('\n', " ")
    }
}
