use crate::admiral::generators::RailLineGenerator;
use crate::admiral::lua_command::{
    FacDestroy, FacLog, FacSurfaceCreateEntity, FacSurfaceCreateEntitySafe, LuaCommand,
};
use opencv::core::{Point, Point2f};
use rcon_client::{AuthRequest, RCONClient, RCONConfig, RCONError, RCONRequest};
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

    fn auth(&mut self) -> Result<(), RCONError> {
        // Auth request to RCON server (SERVERDATA_AUTH)
        let auth_result = self.client.auth(AuthRequest::new("xana".to_string()))?;
        assert!(auth_result.is_success());
        Ok(())
    }

    fn execute_lua(&mut self, lua: impl LuaCommand) -> Result<String, RCONError> {
        let lua_text = lua.make_lua();

        // Execute command request to RCON server (SERVERDATA_EXECCOMMAND)
        let request = RCONRequest::new(format!("/c {}", lua_text));
        debug!("executing\n{}", lua_text);

        let execute = self.client.execute(request)?;
        debug!(
            "id {} type {} body {}",
            execute.id,
            execute.response_type,
            execute.body.len()
        );

        Ok(execute.body)
    }

    fn execute_lua_empty(&mut self, lua: impl LuaCommand) -> Result<(), RCONError> {
        // let lua_text = lua.make_lua();
        match self.execute_lua(lua) {
            Ok(v) => {
                if v.is_empty() {
                    Ok(())
                } else {
                    Err(RCONError::TypeError(format!("not empty")))
                }
            }
            Err(e) => Err(e),
        }
    }

    fn execute_lua_safe(&mut self, lua: impl LuaCommand) -> Result<(), RCONError> {
        let lua_text = lua.make_lua();
        match self.execute_lua(lua) {
            Ok(v) => {
                let v = v.trim();
                if v.is_empty() {
                    Err(RCONError::TypeError(format!(
                        "expected _success metric got empty"
                    )))
                } else if v.ends_with("_success") {
                    Ok(())
                } else {
                    Err(RCONError::TypeError(format!(
                        "expected _success metric got {}",
                        v
                    )))
                }
            }
            Err(e) => Err(e),
        }
    }

    fn log(&mut self, line: &str) -> Result<(), RCONError> {
        info!("[Game Log] {}", line);
        self.execute_lua_empty(FacLog {
            message: line.to_string(),
        })
    }
}

pub fn inner_admiral() -> Result<(), RCONError> {
    let mut admiral = FactoCommands::new()?;

    admiral.auth()?;
    admiral.log("init admiral")?;

    admiral.execute_lua_safe(FacDestroy {})?;

    admiral.execute_lua_empty(RailLineGenerator {
        length: 200,
        rail_loops: 20,
        start: Point2f { x: 1f32, y: 1f32 },
        separator_every_num: 8,
    });

    Ok(())
}

fn _generate_mega_block(admiral: &mut FactoCommands) -> Result<(), RCONError> {
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
    Ok(())
}
