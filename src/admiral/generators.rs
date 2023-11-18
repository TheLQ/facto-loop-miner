use crate::admiral::lua_command::{
    FacSurfaceCreateEntity, FacSurfaceCreateEntitySafe, LuaCommand, DEFAULT_SURFACE_VAR,
};
use opencv::core::{Point, Point2f};
use tracing::debug;

pub struct RailLineGenerator {
    pub(crate) start: Point2f,
    pub(crate) length: u32,
    pub(crate) rail_loops: u32,
    pub(crate) separator_every_num: u32,
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
        for rail_loop in 0..self.rail_loops {
            for length in 0..self.length {
                // let start_x_offset = rail_loop as f32 * 2f32 * 3f32;
                // let mut start_x: f32 = self.start.x + ();
                // debug!("startx {}", start_x);
                // let mut separator_x = start_x as u32 - (rail_loop as u32 % self.separator_every_num);
                // // debug!("separator init {}", separator_x);
                // separator_x = separator_x / self.separator_every_num;
                // start_x = start_x + separator_x as f32;

                let mut start_x = self.start.x;
                debug!("start_x init           {}", start_x);
                start_x = start_x + (rail_loop as f32 * 2f32 * 3f32);
                debug!("start_x with rail_loop {}", start_x);
                start_x = start_x
                    + ((rail_loop - (rail_loop % self.separator_every_num))
                        / self.separator_every_num) as f32;
                debug!("start_x tota rail_loop {}", start_x);

                // debug!("sep {} for {}", separator_x, start_x);

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
