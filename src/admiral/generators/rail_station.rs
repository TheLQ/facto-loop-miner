use crate::admiral::generators::join_commands;
use crate::admiral::generators::rail90::rail_degrees_360;
use crate::admiral::lua_command::{
    direction_params, FacSurfaceCreateEntity, FacSurfaceCreateEntitySafe, LuaCommand,
    DEFAULT_SURFACE_VAR,
};
use opencv::core::Point2f;

pub struct RailStationGenerator {
    pub start: Point2f,
    pub wagon_size: u32,
}

impl LuaCommand for RailStationGenerator {
    fn make_lua(&self) -> String {
        let mut creation_commands: Vec<Box<dyn LuaCommand>> = Vec::new();

        let wagons_to_rails = self.wagon_size * 6;
        let x_end = self.start.x as i32 + wagons_to_rails as i32;

        for y in [self.start.y as i32, self.start.y as i32 + 12] {
            for x in self.start.x as i32..x_end {
                creation_commands.push(Box::new(FacSurfaceCreateEntitySafe {
                    inner: FacSurfaceCreateEntity {
                        name: "straight-rail".to_string(),
                        position: Point2f {
                            x: (x * 2) as f32,
                            y: (y * 2) as f32,
                        },
                        surface_var: DEFAULT_SURFACE_VAR.to_string(),
                        params: direction_params("east"),
                    },
                }));
            }
        }

        rail_degrees_360(Point2f {
            x: self.start.x,
            y: self.start.y,
        })
        .into_iter()
        .for_each(|e| creation_commands.push(e));

        join_commands(creation_commands.iter())
    }
}
