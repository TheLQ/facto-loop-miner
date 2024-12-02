use crate::admiral::generators::xy_grid;
use crate::admiral::lua_command::fac_surface_create_entity::FacSurfaceCreateEntity;
use crate::admiral::lua_command::{LuaCommand, LuaCommandBatch};
use opencv::core::{Point, Point2f};

#[derive(Debug)]
pub struct Terapower {
    pub start: Point,
    pub width: u32,
    pub height: u32,
}

impl LuaCommandBatch for Terapower {
    fn make_lua_batch(self, lua_commands: &mut Vec<Box<dyn LuaCommand>>) {
        for pos in xy_grid(self.start.x, self.start.y, self.width, self.height, 30) {
            lua_commands.push(
                FacSurfaceCreateEntity::new_electric_pole_big(Point2f {
                    // must be odd
                    x: pos.x as f32,
                    y: pos.y as f32,
                })
                .into_boxed(),
            );

            if pos.ix % 6 == 0 && pos.iy % 7 == 6 {
                lua_commands.push(
                    FacSurfaceCreateEntity::new_radar(Point2f {
                        x: pos.x as f32 + 0.5,
                        y: pos.y as f32 + 2.5,
                    })
                    .into_boxed(),
                );
            }
        }
    }
}
