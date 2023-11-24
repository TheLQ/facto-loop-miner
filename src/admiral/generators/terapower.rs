use crate::admiral::generators::xy_grid;
use crate::admiral::lua_command::{
    FacSurfaceCreateEntity, FacSurfaceCreateEntitySafe, LuaCommand, LuaCommandBatch,
    DEFAULT_SURFACE_VAR,
};
use opencv::core::{Point, Point2f};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Terapower {
    pub start: Point,
    pub width: u32,
    pub height: u32,
}

impl LuaCommandBatch for Terapower {
    fn make_lua_batch(self, lua_commands: &mut Vec<Box<dyn LuaCommand>>) {
        for pos in xy_grid(self.start.x, self.start.y, self.width, self.height, 30) {
            lua_commands.push(Box::new(FacSurfaceCreateEntitySafe {
                inner: FacSurfaceCreateEntity {
                    name: "big-electric-pole".to_string(),
                    params: HashMap::new(),
                    position: Point2f {
                        x: pos.x as f32,
                        y: pos.y as f32,
                    },
                    surface_var: DEFAULT_SURFACE_VAR.to_string(),
                    extra: Vec::new(),
                },
            }));

            if pos.ix % 6 == 0 && pos.iy % 7 == 6 {
                // if true {
                lua_commands.push(Box::new(FacSurfaceCreateEntitySafe {
                    inner: FacSurfaceCreateEntity {
                        name: "radar".to_string(),
                        params: HashMap::new(),
                        position: Point2f {
                            x: pos.x as f32 + 0.5,
                            y: pos.y as f32 + 2.5,
                        },
                        surface_var: DEFAULT_SURFACE_VAR.to_string(),
                        extra: Vec::new(),
                    },
                }));
            }
        }
    }
}
