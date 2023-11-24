use crate::admiral::err::{AdmiralError, AdmiralResult};
use crate::admiral::executor::ExecuteResponse;
use crate::admiral::executor::LuaCompiler;
use crate::admiral::lua_command::fac_execution_define::FacExectionDefine;
use crate::admiral::lua_command::LuaCommand;
use std::backtrace::Backtrace;
use std::fs::File;
use std::io::Write;
use tracing::debug;

pub struct AdmiralFile {
    path: String,
    pub output_file: File,
}

impl AdmiralFile {
    pub fn new() -> AdmiralResult<Self> {
        let path = "/home/desk/factorio/mods/megacalc/megacall_auto.lua".to_string();
        Ok(AdmiralFile {
            path: path.clone(),
            output_file: File::open(path.clone()).map_err(|e| AdmiralError::IoError {
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
            body: "luafile_success".to_string(),
            lua,
        })
    }

    fn _execute_define(
        &mut self,
        lua_define: FacExectionDefine,
    ) -> AdmiralResult<ExecuteResponse<FacExectionDefine>> {
        let lua_text = lua_define.make_lua();
        self.output_file
            .write_all(lua_text.as_bytes())
            .map_err(|e| AdmiralError::IoError {
                e,
                backtrace: Backtrace::capture(),
                path: self.path.clone(),
            });
        debug!("wrote {} bytes", lua_text.len());

        Ok(ExecuteResponse {
            lua_text: "".to_string(),
            body: "facexecution_define".to_string(),
            lua: lua_define,
        })
    }
}
