use crate::admiral::err::{AdmiralError, AdmiralResult};
use crate::admiral::generators::beacon_farm::BeaconFarmGenerator;
use crate::admiral::generators::rail_beacon_farm::RailBeaconFarmGenerator;
use crate::admiral::generators::rail_line::RailLineGenerator;
use crate::admiral::generators::rail_station::RailStationGenerator;
use crate::admiral::lua_command::{
    FacDestroy, FacExectionDefine, FacExectionRun, FacLog, FacSurfaceCreateEntity,
    FacSurfaceCreateEntitySafe, LuaCommand,
};
use crate::surface::metric::Metrics;
use num_format::Grouping::Posix;
use opencv::core::{Point, Point2f};
use rcon_client::{AuthRequest, RCONClient, RCONConfig, RCONError, RCONRequest};
use std::backtrace::Backtrace;
use std::collections::HashMap;
use std::fmt::format;
use tracing::{debug, info};

pub fn admiral() {
    inner_admiral().unwrap()
}

struct FactoCommands {
    client: RCONClient,
}

impl FactoCommands {
    fn new() -> Result<Self, RCONError> {
        let client = RCONClient::new(RCONConfig {
            url: "192.168.66.73:28016".to_string(),
            // Optional
            read_timeout: Some(13),
            write_timeout: Some(37),
        })?;

        Ok(FactoCommands { client })
    }

    fn auth(&mut self) -> AdmiralResult<()> {
        // Auth request to RCON server (SERVERDATA_AUTH)
        let auth_result = self.client.auth(AuthRequest::new("xana".to_string()))?;
        assert!(auth_result.is_success());
        Ok(())
    }

    fn execute_lua(&mut self, lua: impl LuaCommand) -> AdmiralResult<String> {
        let lua_text = lua.make_lua();
        if lua_text.trim().is_empty() {
            return Err(AdmiralError::LuaBlankCommand {
                backtrace: Backtrace::capture(),
            });
        }

        // Execute command request to RCON server (SERVERDATA_EXECCOMMAND)
        let request = RCONRequest::new(format!("/c {}", lua_text));
        // debug!("executing\n{}", lua_text);

        let execute = self
            .client
            .execute(request)
            .map_err(|e| AdmiralError::Rcon {
                source: e,
                backtrace: Backtrace::capture(),
            })?;
        debug!(
            "id {} type {} body {}",
            execute.id,
            execute.response_type,
            execute.body.len()
        );

        Ok(execute.body)
    }

    fn execute_lua_empty(&mut self, lua: impl LuaCommand) -> AdmiralResult<()> {
        // let lua_text = lua.make_lua();
        self.execute_lua(lua).and_then(|v| {
            if v.is_empty() {
                Ok(())
            } else {
                Err(AdmiralError::LuaResultNotEmpty {
                    body: v,
                    backtrace: Backtrace::capture(),
                })
            }
        })
    }

    fn execute_lua_safe(&mut self, lua: impl LuaCommand) -> AdmiralResult<()> {
        let lua_text = lua.make_lua();
        self.execute_lua_empty(FacExectionDefine { body: lua_text })?;

        self.log("starting megacall...")?;

        let lua_text = "megacall()";
        self.execute_lua(FacExectionRun {}).and_then(|v| {
            let v = v.trim();
            if v.is_empty() {
                return Err(AdmiralError::LuaResultEmpty {
                    backtrace: Backtrace::capture(),
                });
            }
            let mut metric = Metrics::new("ExecuteResult".to_string());
            for part in v.split("\n") {
                metric.increment(part);
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
        self.execute_lua_empty(FacLog {
            message: line.to_string(),
        })
    }
}

pub fn inner_admiral() -> AdmiralResult<()> {
    let mut admiral = FactoCommands::new()?;

    admiral.auth()?;
    admiral.log("init admiral")?;

    admiral.execute_lua_safe(FacDestroy {})?;

    let res = admiral.execute_lua_safe(RailStationGenerator {
        wagon_size: 8,
        start: Point2f { x: 200.0, y: 200.0 },
    })?;

    Ok(())
}

fn _generate_mega_block(admiral: &mut FactoCommands) -> AdmiralResult<()> {
    for x in 0..50 {
        for y in 0..50 {
            let text = admiral.execute_lua_safe(FacSurfaceCreateEntitySafe {
                inner: FacSurfaceCreateEntity {
                    surface_var: "game.surfaces[1]".to_string(),
                    position: Point2f::new(1f32 + (x as f32 * 2.0), 1f32 + (y as f32 * 2.0)),
                    name: "straight-rail".to_string(),
                    params: HashMap::new(),
                },
            })?;
        }
    }

    admiral.execute_lua_empty(RailLineGenerator {
        length: 200,
        rail_loops: 20,
        start: Point2f { x: 1f32, y: 1f32 },
        separator_every_num: 8,
    })?;

    admiral.execute_lua_safe(RailBeaconFarmGenerator {
        inner: BeaconFarmGenerator {
            cell_size: 3,
            width: 20,
            height: 15,
            start: Point2f { x: 200.5, y: 200.5 },
        },
    })?;

    Ok(())
}
