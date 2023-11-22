use crate::admiral::lua_command::{
    direction_params, FacSurfaceCreateEntity, FacSurfaceCreateEntitySafe, LuaCommand,
    LuaCommandBatch, DEFAULT_SURFACE_VAR,
};
use opencv::core::Point2f;

#[derive(Debug)]
pub struct RailStationPathfoundGenerator {
    pub start: Point2f,
    pub station: Point2f,
    pub pan: Point2f,
}

impl LuaCommandBatch for RailStationPathfoundGenerator {
    fn make_lua_batch(self, lua_commands: &mut Vec<Box<dyn LuaCommand>>) {
        lua_commands.push(Box::new(FacSurfaceCreateEntitySafe {
            inner: FacSurfaceCreateEntity {
                name: "straight-rail".to_string(),
                position: Point2f {
                    x: self.start.x,
                    y: self.start.y,
                },
                surface_var: DEFAULT_SURFACE_VAR.to_string(),
                extra: Vec::new(),
                params: direction_params("east"),
            },
        }));

        lua_commands.push(Box::new(FacSurfaceCreateEntitySafe {
            inner: FacSurfaceCreateEntity {
                name: "straight-rail".to_string(),
                position: Point2f {
                    x: self.station.x,
                    y: self.station.y,
                },
                surface_var: DEFAULT_SURFACE_VAR.to_string(),
                extra: Vec::new(),
                params: direction_params("east"),
            },
        }));

        lua_commands.push(Box::new(FacSurfaceCreateEntitySafe {
            inner: FacSurfaceCreateEntity {
                name: "straight-rail".to_string(),
                position: Point2f {
                    x: self.pan.x,
                    y: self.pan.y,
                },
                surface_var: DEFAULT_SURFACE_VAR.to_string(),
                extra: Vec::new(),
                params: direction_params("east"),
            },
        }));
    }
}
