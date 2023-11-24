use crate::admiral::lua_command::fac_surface_create_entity::FacSurfaceCreateEntity;
use crate::admiral::lua_command::fac_surface_create_entity_safe::FacSurfaceCreateEntitySafe;
use crate::admiral::lua_command::{LuaCommand, LuaCommandBatch, DEFAULT_SURFACE_VAR};
use crate::admiral::must_half_number;
use opencv::core::Point2f;
use std::collections::HashMap;

pub const BEACON_SIZE: u32 = 3;

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
        must_half_number(self.start);
        let zero_cell_size = self.cell_size - 1;
        for x in 0..(zero_cell_size * self.width) + 1 {
            for y in 0..(zero_cell_size * self.height) + 1 {
                if y % zero_cell_size == 0 || x % zero_cell_size == 0 {
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
