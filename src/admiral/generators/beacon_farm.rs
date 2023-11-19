use crate::admiral::lua_command::{
    FacSurfaceCreateEntity, FacSurfaceCreateEntitySafe, LuaCommand, DEFAULT_SURFACE_VAR,
};
use crate::gamedata::lua::LuaEntity;
use opencv::core::{Point2f, Point_};
use std::collections::HashMap;

#[derive(Debug)]
pub struct BeaconFarmGenerator {
    pub start: Point2f,
    pub cell_size: u32,
    pub width: u32,
    pub height: u32,
}

impl LuaCommand for BeaconFarmGenerator {
    fn make_lua(&self) -> String {
        let mut lua_commands: Vec<Box<dyn LuaCommand>> = Vec::new();
        // TODO: why is magic 1 needed
        for x in 0..(self.cell_size * self.width + 1) {
            for y in 0..(self.cell_size * self.height + 1) {
                if y % self.cell_size == 0 || x % self.cell_size == 0 {
                    lua_commands.push(Box::new(FacSurfaceCreateEntitySafe {
                        inner: FacSurfaceCreateEntity {
                            name: "beacon".to_string(),
                            params: HashMap::new(),
                            position: Point2f {
                                x: self.start.x + x as f32 * 3f32,
                                y: self.start.y + y as f32 * 3f32,
                            },
                            surface_var: DEFAULT_SURFACE_VAR.to_string(),
                        },
                    }));
                }
            }
        }

        let mut result = "".to_string();
        for command in lua_commands {
            result.push_str(&command.make_lua());
        }
        result
    }
}
