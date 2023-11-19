use crate::admiral::err::{AdmiralError, AdmiralResult};
use crate::admiral::generators::beacon_farm::BeaconFarmGenerator;
use crate::admiral::generators::rail_beacon_farm::RailBeaconFarmGenerator;
use crate::admiral::generators::rail_line::RailLineGenerator;
use crate::admiral::generators::rail_station::RailStationGenerator;
use crate::admiral::lua_command::{
    BasicLuaBatch, FacDestroy, FacExectionDefine, FacExectionRun, FacLog, FacSurfaceCreateEntity,
    FacSurfaceCreateEntitySafe, LuaCommand, LuaCommandBatch,
};
use crate::surface::metric::Metrics;
use num_format::Grouping::Posix;
use opencv::core::{Point, Point2f};
use rcon_client::{AuthRequest, RCONClient, RCONConfig, RCONError, RCONRequest};
use std::backtrace::Backtrace;
use std::collections::HashMap;
use std::fmt::{format, Debug};
use tracing::{debug, error, info, warn};

pub fn admiral() {
    if let Err(e) = inner_admiral() {
        error!("Admiral failed! {:#?}", e);
    }
}

struct AdmiralClient {
    client: RCONClient,
}

impl AdmiralClient {
    fn new() -> Result<Self, RCONError> {
        let client = RCONClient::new(RCONConfig {
            url: "192.168.66.73:28016".to_string(),
            // Optional
            read_timeout: Some(13),
            write_timeout: Some(37),
        })?;

        Ok(AdmiralClient { client })
    }

    fn auth(&mut self) -> AdmiralResult<()> {
        // Auth request to RCON server (SERVERDATA_AUTH)
        let auth_result = self.client.auth(AuthRequest::new("xana".to_string()))?;
        assert!(auth_result.is_success());
        Ok(())
    }

    fn _execute_statement<L>(&mut self, lua: L) -> AdmiralResult<(String, L)>
    where
        L: LuaCommand,
    {
        let lua_text = lua.make_lua();
        if lua_text.trim().is_empty() {
            return Err(AdmiralError::LuaBlankCommand {
                backtrace: Backtrace::capture(),
            });
        }

        // Execute command request to RCON server (SERVERDATA_EXECCOMMAND)
        let request = RCONRequest::new(format!("/c {}", lua_text));
        debug!("executing\n{}", lua_text);

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

        Ok((execute.body, lua))
    }

    fn _execute_statement_empty(&mut self, lua: impl LuaCommand + Debug) -> AdmiralResult<()> {
        self._execute_statement(lua).and_then(|(v, lua)| {
            if v.is_empty() {
                Ok(())
            } else {
                Err(AdmiralError::LuaResultNotEmpty {
                    command: format!("{:#?}", lua),
                    body: v,
                    backtrace: Backtrace::capture(),
                })
            }
        })
    }

    fn execute_block(&mut self, lua: impl LuaCommandBatch + Debug) -> AdmiralResult<()> {
        let commands = lua.make_lua_batch();
        let command_num = commands.len();
        debug!("Execute Block with {} commands", command_num);
        self._execute_statement_empty(FacExectionDefine { commands })?;

        let lua_text = "megacall()";
        self._execute_statement(FacExectionRun {})
            .and_then(|(v, lua)| {
                let v = v.trim();
                if v.is_empty() {
                    return Err(AdmiralError::LuaResultEmpty {
                        command: format!("{:?}", lua),
                        backtrace: Backtrace::capture(),
                    });
                }
                let mut metric = Metrics::new("ExecuteResult".to_string());
                for part in v.split("\n") {
                    if part.contains(" ") {
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

    fn log(&mut self, line: &str) -> AdmiralResult<()> {
        info!("[Game Log] {}", line);
        self._execute_statement_empty(FacLog {
            message: line.to_string(),
        })
    }
}

pub fn inner_admiral() -> AdmiralResult<()> {
    let mut admiral = AdmiralClient::new()?;

    admiral.auth()?;
    admiral.log("init admiral")?;

    admiral.execute_block(BasicLuaBatch {
        commands: vec![Box::new(FacDestroy {})],
    })?;

    let res = admiral.execute_block(RailStationGenerator {
        wagon_size: 8,
        start: Point2f { x: 200.0, y: 200.0 },
    })?;

    Ok(())
}

fn _generate_mega_block(admiral: &mut AdmiralClient) -> AdmiralResult<()> {
    // for x in 0..50 {
    //     for y in 0..50 {
    //         let text = admiral.execute_block(FacSurfaceCreateEntitySafe {
    //             inner: FacSurfaceCreateEntity {
    //                 surface_var: "game.surfaces[1]".to_string(),
    //                 position: Point2f::new(1f32 + (x as f32 * 2.0), 1f32 + (y as f32 * 2.0)),
    //                 name: "straight-rail".to_string(),
    //                 params: HashMap::new(),
    //             },
    //         })?;
    //     }
    // }
    //
    // admiral.execute_lua_empty(RailLineGenerator {
    //     length: 200,
    //     rail_loops: 20,
    //     start: Point2f { x: 1f32, y: 1f32 },
    //     separator_every_num: 8,
    // })?;
    //
    // admiral.execute_block(RailBeaconFarmGenerator {
    //     inner: BeaconFarmGenerator {
    //         cell_size: 3,
    //         width: 20,
    //         height: 15,
    //         start: Point2f { x: 200.5, y: 200.5 },
    //     },
    // })?;

    Ok(())
}
