use crate::admiral::err::{AdmiralError, AdmiralResult};
use crate::admiral::lua_command::LuaCommand;
use crate::admiral::lua_command::checked_command::CheckedLuaCommand;
use crate::admiral::lua_command::fac_surface_create_entity::{
    DEBUG_POSITION_EXPECTED, DEBUG_PRE_COLLISION,
};
use crate::admiral::lua_command::lua_batch::LuaBatchCommand;
use itertools::Itertools;
use std::backtrace::Backtrace;

// pub mod client;
// pub mod entrypoint;
// pub mod file;

const BATCH_SIZE: usize = if DEBUG_POSITION_EXPECTED || DEBUG_PRE_COLLISION {
    // max lua variables per function
    200
} else {
    // effectively infinite?
    200_000
};

pub trait LuaCompiler {
    fn _execute_statement(&mut self, lua: impl LuaCommand) -> AdmiralResult<ExecuteResponse>;

    fn execute_checked_command(
        &mut self,
        lua: Box<dyn LuaCommand>,
    ) -> AdmiralResult<ExecuteResponse> {
        let checked = CheckedLuaCommand::new(lua);
        let checked_id = checked.id();
        let res = self._execute_statement(checked)?;

        let body = res.body.trim();
        if body.is_empty() {
            Err(AdmiralError::LuaCheckedEmpty {
                command: res.lua_text,
                backtrace: Backtrace::capture(),
            })
        } else if body != checked_id.to_string() {
            if body.contains("[Admiral]") {
                Err(AdmiralError::LuaCheckedError {
                    command: res.lua_text,
                    errors: res.body,
                    backtrace: Backtrace::capture(),
                })
            } else {
                Err(AdmiralError::LuaCheckedUnknown {
                    command: res.lua_text,
                    body: res.body,
                    backtrace: Backtrace::capture(),
                })
            }
        } else {
            Ok(res)
        }
    }

    fn execute_checked_commands_in_wrapper_function(
        &mut self,
        commands: Vec<Box<dyn LuaCommand>>,
    ) -> AdmiralResult<()> {
        for batch in &commands.into_iter().chunks(BATCH_SIZE) {
            let command = LuaBatchCommand::new(batch.collect()).into_boxed();
            self.execute_checked_command(command)?;
        }

        Ok(())
    }

    // fn _execute_statement_empty(&mut self, lua: impl LuaCommand) -> AdmiralResult<()> {
    //     self._execute_statement(lua).and_then(|response| {
    //         if response.body.is_empty() {
    //             Ok(())
    //         } else {
    //             Err(AdmiralError::LuaResultNotEmpty {
    //                 command: format!("{:#?}", response.lua),
    //                 body: response.body,
    //                 backtrace: Backtrace::capture(),
    //             })
    //         }
    //     })
    // }
    //
    // fn _execute_define(
    //     &mut self,
    //     lua_define: FacExectionDefine,
    // ) -> AdmiralResult<ExecuteResponse<FacExectionDefine>> {
    //     self._execute_statement(lua_define)
    // }
    //
    // fn execute_block(&mut self, lua: impl LuaCommandBatch) -> AdmiralResult<()> {
    //     let mut commands: Vec<Box<dyn LuaCommand>> = Vec::new();
    //     info!("Executing {:?}", lua);
    //     lua.make_lua_batch(&mut commands);
    //     let command_num = commands.len();
    //     debug!("Execute Block with {} commands", command_num);
    //     self._execute_define(FacExectionDefine { commands })
    //         .and_then(|response| {
    //             let v = response.body.trim();
    //             if v.is_empty() {
    //                 Err(AdmiralError::DefineFailed {
    //                     lua_text: response.lua_text,
    //                     backtrace: Backtrace::capture(),
    //                 })
    //             } else if v == "facexecution_define" {
    //                 trace!("succesfully defined ");
    //                 Ok(())
    //             } else {
    //                 Err(AdmiralError::LuaResultNotEmpty {
    //                     command: format!("{:?}", response.lua),
    //                     body: v.to_string(),
    //                     backtrace: Backtrace::capture(),
    //                 })
    //             }
    //         })?;
    //
    //     let lua_text = "megacall()";
    //     self._execute_statement(FacExectionRun {})
    //         .and_then(|response| {
    //             let v = response.body.trim();
    //             if v.is_empty() {
    //                 return Err(AdmiralError::LuaResultEmpty {
    //                     command: format!("{:?}", response.lua),
    //                     backtrace: Backtrace::capture(),
    //                 });
    //             }
    //             let mut metric = Metrics::new("ExecuteResult");
    //             for part in v.split('\n') {
    //                 if part.contains(' ') {
    //                     warn!("[lua_log] {}", part);
    //                 } else {
    //                     metric.increment_slow(part);
    //                 }
    //                 // if !v.ends_with("_success") {
    //                 //     return Err(RCONError::TypeError(format!(
    //                 //         "expected _success metric got {}",
    //                 //         v
    //                 //     )));
    //                 // }
    //             }
    //             metric.log_final();
    //
    //             Ok(())
    //         })
    // }
}

pub struct ExecuteResponse {
    pub body: String,
    pub lua_text: String,
}
