use crate::admiral::lua_command::fac_surface_create_entity::FacSurfaceCreateEntity;
use crate::admiral::lua_command::{LuaCommand, LuaCommandBatch};
use crate::surfacev::vpoint::must_half_number;
use opencv::core::Point2f;

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
                    lua_commands.push(
                        FacSurfaceCreateEntity::new_commands(
                            "beacon",
                            Point2f {
                                x: self.start.x + x as f32 * 3f32,
                                y: self.start.y + y as f32 * 3f32,
                            },
                            vec![format!(
                                "admiral_create.get_module_inventory().insert(\"{}\")",
                                self.module
                            )],
                        )
                        .into_boxed(),
                    );
                }
            }
        }
    }
}
