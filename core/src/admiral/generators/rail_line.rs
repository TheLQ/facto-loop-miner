use crate::admiral::lua_command::fac_surface_create_entity::FacSurfaceCreateEntity;
use crate::admiral::lua_command::LuaCommand;
use crate::navigator::mori::RailDirection;
use opencv::core::Point2f;
use tracing::debug;

#[derive(Debug)]
pub struct RailLineGenerator {
    pub(crate) start: Point2f,
    pub(crate) length: u32,
    pub(crate) rail_loops: u32,
    pub(crate) separator_every_num: u32,
}

impl LuaCommand for RailLineGenerator {
    fn make_lua(&self) -> String {
        assert_ne!(
            self.start.x % 2f32,
            1f32,
            "invalid x start {}",
            self.start.x
        );
        assert_ne!(
            self.start.y % 2f32,
            1f32,
            "invalid y start {}",
            self.start.y
        );

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

                if rail_loop % self.separator_every_num == 0 {
                    continue;
                }

                let start_x = self.start.x + (rail_loop as f32 * 2f32 * 3f32);
                debug!("start_x tota rail_loop {}", start_x);

                // debug!("sep {} for {}", separator_x, start_x);

                let start_y: f32 = self.start.y + length as f32;

                creation_commands.push(
                    FacSurfaceCreateEntity::new_rail_straight(
                        Point2f {
                            x: start_x,
                            y: start_y,
                        },
                        RailDirection::Down,
                    )
                    .into_boxed(),
                );

                creation_commands.push(
                    FacSurfaceCreateEntity::new_rail_straight(
                        Point2f {
                            x: start_x + 4.0,
                            y: start_y,
                        },
                        RailDirection::Down,
                    )
                    .into_boxed(),
                );

                if length % 32 == 0 {
                    creation_commands.push(
                        FacSurfaceCreateEntity::new_rail_signal(
                            Point2f {
                                x: start_x + 1.5,
                                y: start_y - 0.5,
                            },
                            RailDirection::Down,
                        )
                        .into_boxed(),
                    );

                    creation_commands.push(
                        FacSurfaceCreateEntity::new_rail_signal(
                            Point2f {
                                x: start_x + 2.5,
                                y: start_y - 0.5,
                            },
                            RailDirection::Up,
                        )
                        .into_boxed(),
                    );
                }
            }
        }

        debug!("generated {} rails", creation_commands.len());

        let mut result = "function railgen()".to_string();

        result.push_str("end\n");
        result.push_str("railgen()");
        result
    }
}
