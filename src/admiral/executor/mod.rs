use crate::admiral::err::{AdmiralError, AdmiralResult};
use crate::admiral::lua_command::fac_execution_define::FacExectionDefine;
use crate::admiral::lua_command::fac_execution_run::FacExectionRun;
use crate::admiral::lua_command::{LuaCommand, LuaCommandBatch};
use crate::surface::metric::Metrics;
use std::backtrace::Backtrace;
use std::fmt::Debug;
use tracing::{debug, info, trace, warn};

pub mod file;
pub mod rcon;

pub trait LuaCompiler {
    fn _execute_statement<L: LuaCommand>(&mut self, lua: L) -> AdmiralResult<ExecuteResponse<L>>;

    fn _execute_statement_empty(&mut self, lua: impl LuaCommand + Debug) -> AdmiralResult<()> {
        self._execute_statement(lua).and_then(|response| {
            if response.body.is_empty() {
                Ok(())
            } else {
                Err(AdmiralError::LuaResultNotEmpty {
                    command: format!("{:#?}", response.lua),
                    body: response.body,
                    backtrace: Backtrace::capture(),
                })
            }
        })
    }

    fn _execute_define(
        &mut self,
        lua_define: FacExectionDefine,
    ) -> AdmiralResult<ExecuteResponse<FacExectionDefine>> {
        self._execute_statement(lua_define)
    }

    fn execute_block(&mut self, lua: impl LuaCommandBatch + Debug) -> AdmiralResult<()> {
        let mut commands: Vec<Box<dyn LuaCommand>> = Vec::new();
        info!("Executing {:?}", lua);
        lua.make_lua_batch(&mut commands);
        let command_num = commands.len();
        debug!("Execute Block with {} commands", command_num);
        self._execute_define(FacExectionDefine { commands })
            .and_then(|response| {
                let v = response.body.trim();
                if v.is_empty() {
                    Err(AdmiralError::DefineFailed {
                        lua_text: response.lua_text,
                        backtrace: Backtrace::capture(),
                    })
                } else if v == "facexecution_define" {
                    trace!("succesfully defined ");
                    Ok(())
                } else {
                    Err(AdmiralError::LuaResultNotEmpty {
                        command: format!("{:?}", response.lua),
                        body: v.to_string(),
                        backtrace: Backtrace::capture(),
                    })
                }
            })?;

        let lua_text = "megacall()";
        self._execute_statement(FacExectionRun {})
            .and_then(|response| {
                let v = response.body.trim();
                if v.is_empty() {
                    return Err(AdmiralError::LuaResultEmpty {
                        command: format!("{:?}", response.lua),
                        backtrace: Backtrace::capture(),
                    });
                }
                let mut metric = Metrics::new("ExecuteResult");
                for part in v.split('\n') {
                    if part.contains(' ') {
                        warn!("[lua_log] {}", part);
                    } else {
                        metric.increment(part);
                    }
                    // if !v.ends_with("_success") {
                    //     return Err(RCONError::TypeError(format!(
                    //         "expected _success metric got {}",
                    //         v
                    //     )));
                    // }
                }
                metric.log_final();

                Ok(())
            })
    }
}

pub struct ExecuteResponse<L: LuaCommand> {
    pub body: String,
    pub lua: L,
    pub lua_text: String,
}
