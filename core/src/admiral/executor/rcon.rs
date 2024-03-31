use crate::admiral::err::{AdmiralError, AdmiralResult};
use crate::admiral::executor::ExecuteResponse;
use crate::admiral::executor::LuaCompiler;
use crate::admiral::lua_command::fac_log::FacLog;
use crate::admiral::lua_command::LuaCommand;
use crate::LOCALE;
use num_format::ToFormattedString;
use rcon_client::{AuthRequest, RCONClient, RCONConfig, RCONRequest};
use std::backtrace::Backtrace;
use tracing::{debug, info, trace};

pub struct AdmiralClient {
    client: RCONClient,
}

impl AdmiralClient {
    pub fn new() -> AdmiralResult<Self> {
        let client = RCONClient::new(RCONConfig {
            url: "127.0.0.1:28016".to_string(),
            // Optional
            // read_timeout: Some(900),
            // write_timeout: Some(900),
            // read_timeout: None,
            // write_timeout: None,
            read_timeout: Some(u64::MAX),
            write_timeout: Some(u64::MAX),
        })
        .map_err(|e| AdmiralError::Rcon {
            source: e,
            backtrace: Backtrace::capture(),
        })?;

        Ok(AdmiralClient { client })
    }

    pub fn auth(&mut self) -> AdmiralResult<()> {
        // Auth request to RCON server (SERVERDATA_AUTH)
        let auth_result = self.client.auth(AuthRequest::new("xana".to_string()))?;
        assert!(auth_result.is_success());
        Ok(())
    }

    pub fn log(&mut self, line: &str) -> AdmiralResult<()> {
        info!("[Game Log] {}", line);
        self._execute_statement_empty(FacLog {
            message: line.to_string(),
        })
    }
}

impl LuaCompiler for AdmiralClient {
    fn _execute_statement<L>(&mut self, lua: L) -> AdmiralResult<ExecuteResponse<L>>
    where
        L: LuaCommand,
    {
        let lua_text = lua.make_lua();
        if lua_text.trim().is_empty() {
            return Err(AdmiralError::LuaBlankCommand {
                backtrace: Backtrace::capture(),
            });
        };
        trace!("Characters {}", lua_text.len().to_formatted_string(&LOCALE));
        if lua_text.len() >= 100 * 1000 * 1000 {
            return Err(AdmiralError::TooLargeRequest {
                lua_text,
                backtrace: Backtrace::capture(),
            });
        }

        // Execute command request to RCON server (SERVERDATA_EXECCOMMAND)
        let request = RCONRequest::new(format!("/c {}", lua_text));
        // let request = RCONRequest::new("/h".to_string());
        // let request = RCONRequest {
        //     id: 5,
        //     body: "/h".to_string(),
        //     request_type: 2,
        // };
        debug!("executing {:?}", request);

        let execute = self
            .client
            .execute(request)
            .map_err(|e| AdmiralError::Rcon {
                source: e,
                backtrace: Backtrace::capture(),
            })?;
        debug!(
            "Execute Result id {} type {} body {}",
            execute.id,
            execute.response_type,
            execute.body.len()
        );

        // Ok((execute.body, lua, lua_text))
        Ok(ExecuteResponse {
            lua_text,
            lua,
            body: execute.body,
        })
    }
}
