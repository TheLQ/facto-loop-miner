use crate::admiral::lua_command::{
    FacSurfaceCreateEntity, FacSurfaceCreateEntitySafe, LuaCommand, DEFAULT_SURFACE_VAR,
};
use opencv::core::{Point, Point2f};
use tracing::debug;

pub struct RailLineGenerator {
    pub(crate) start: Point2f,
    pub(crate) length: u32,
    pub(crate) lines: u8,
    pub(crate) separator_every_num: u8,
}

impl LuaCommand for RailLineGenerator {
    fn make_lua(&self) -> String {
        if self.start.x % 2f32 != 1f32 {
            panic!("invalid x start {}", self.start.x)
        }
        if self.start.y % 2f32 != 1f32 {
            panic!("invalid y start {}", self.start.y)
        }

        let mut creation_commands: Vec<Box<dyn LuaCommand>> = Vec::new();
        for line in 0..self.lines {
            for length in 0..self.length {
                let start_x: f32 = self.start.x + (line as f32 * 2f32 * 3f32);
                let start_y: f32 = self.start.y + length as f32;

                creation_commands.push(Box::new(FacSurfaceCreateEntitySafe {
                    inner: FacSurfaceCreateEntity {
                        name: "straight-rail".to_string(),
                        position: Point2f {
                            x: start_x,
                            y: start_y,
                        },
                        surface_var: DEFAULT_SURFACE_VAR.to_string(),
                    },
                }));

                creation_commands.push(Box::new(FacSurfaceCreateEntitySafe {
                    inner: FacSurfaceCreateEntity {
                        name: "straight-rail".to_string(),
                        position: Point2f {
                            x: start_x + 4.0,
                            y: start_y,
                        },
                        surface_var: DEFAULT_SURFACE_VAR.to_string(),
                    },
                }));
            }
        }

        debug!("generated {} rails", creation_commands.len());

        let mut result = "function railgen()".to_string();
        for command in creation_commands {
            result.push_str(&command.make_lua());
            result.push('\n');
        }
        result.push_str("end\n");
        result.push_str("railgen()");
        result
    }
}
