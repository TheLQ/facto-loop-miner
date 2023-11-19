use crate::admiral::lua_command::{
    FacSurfaceCreateEntity, FacSurfaceCreateEntitySafe, LuaCommand, LuaCommandBatch,
    DEFAULT_SURFACE_VAR,
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
    pub module: String,
}

impl LuaCommandBatch for BeaconFarmGenerator {
    fn make_lua_batch(self, lua_commands: &mut Vec<Box<dyn LuaCommand>>) {
        // TODO: why is magic 1 needed
        for x in 0..(self.cell_size * self.width) {
            for y in 0..(self.cell_size * self.height) {
                if y % (self.cell_size - 1) == 0 || x % (self.cell_size - 1) == 0 {
                    lua_commands.push(Box::new(FacSurfaceCreateEntitySafe {
                        inner: FacSurfaceCreateEntity {
                            name: "beacon".to_string(),
                            params: HashMap::new(),
                            position: Point2f {
                                x: self.start.x + x as f32 * 3f32,
                                y: self.start.y + y as f32 * 3f32,
                            },
                            surface_var: DEFAULT_SURFACE_VAR.to_string(),
                            extra: vec![format!(
                                "admiral_create.get_module_inventory().insert(\"{}\")",
                                self.module
                            )],
                        },
                    }));
                }
            }
        }
    }
}
