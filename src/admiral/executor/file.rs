use crate::admiral::err::{AdmiralError, AdmiralResult};
use crate::admiral::lua_command::compiler::{ExecuteResponse, LuaCompiler};
use crate::admiral::lua_command::LuaCommand;
use rcon_client::RCONClient;
use std::backtrace::Backtrace;
use std::fs::File;

pub struct AdmiralFile {
    pub output_file: File,
}

impl AdmiralFile {
    fn new() -> AdmiralResult<Self> {
        let path = "/home/desk/factorio/mods/megacalc/megacall_auto.lua";
        Ok(AdmiralFile {
            output_file: File::open(path).map_err(|e| AdmiralError::IoError {
                e,
                backtrace: Backtrace::capture(),
                path: path.to_string(),
            })?,
        })
    }
}

impl LuaCompiler for AdmiralFile {
    fn _execute_statement<L: LuaCommand>(&mut self, lua: L) -> AdmiralResult<ExecuteResponse<L>> {
        Ok(ExecuteResponse {
            lua_text: "".to_string(),
            body: "".to_string(),
            lua,
        })
    }
}
