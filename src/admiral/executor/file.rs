use crate::admiral::err::{AdmiralError, AdmiralResult};
use crate::admiral::executor::ExecuteResponse;
use crate::admiral::executor::LuaCompiler;
use crate::admiral::lua_command::fac_execution_define::FacExectionDefine;
use crate::admiral::lua_command::LuaCommand;
use std::backtrace::Backtrace;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::Write;
use tracing::debug;

pub struct AdmiralFile {
    path: String,
    pub output_file: File,
}

impl AdmiralFile {
    pub fn new() -> AdmiralResult<Self> {
        let path = "/home/desk/factorio/mods/megacalc/megacall_auto.lua".to_string();
        let mut output_file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(path.clone())
            .map_err(|e| AdmiralError::IoError {
                e,
                backtrace: Backtrace::capture(),
                path: path.to_string(),
            })?;
        let mut admiral = AdmiralFile {
            path: path.clone(),
            output_file,
        };

        admiral
            .output_file
            .write_all("local function autowrapper()".as_bytes())
            .map_err(|e| admiral.new_io_error(e))?;

        Ok(admiral)
    }

    pub fn end_file(&mut self) -> AdmiralResult<()> {
        self.output_file
            .write_all("end\nreturn { mega = autowrapper }\n".as_bytes())
            .map_err(|e| self.new_io_error(e))
    }

    fn new_io_error(&self, e: io::Error) -> AdmiralError {
        AdmiralError::IoError {
            e,
            backtrace: Backtrace::capture(),
            path: self.path.to_string(),
        }
    }
}

impl LuaCompiler for AdmiralFile {
    fn _execute_statement<L: LuaCommand>(&mut self, lua: L) -> AdmiralResult<ExecuteResponse<L>> {
        let mut lua_text = lua.make_lua();
        lua_text = lua_text.replace("rcon.print", "log");
        self.output_file
            .write_all(lua_text.as_bytes())
            .map_err(|e| self.new_io_error(e))?;
        debug!("wrote {} bytes", lua_text.len());

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
        LuaCompiler::_execute_statement(self, lua_define).map(|r| ExecuteResponse {
            lua_text: "".to_string(),
            body: "facexecution_define".to_string(),
            lua: r.lua,
        })
    }
}
