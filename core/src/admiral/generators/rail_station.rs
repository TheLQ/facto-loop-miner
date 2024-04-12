use crate::admiral::generators::rail90::{rail_degrees_east, rail_degrees_north};
use crate::admiral::lua_command::fac_surface_create_entity::{CreateParam, FacSurfaceCreateEntity};
use crate::admiral::lua_command::{LuaCommand, LuaCommandBatch};
use crate::navigator::mori::RailDirection;
use crate::surfacev::vpoint::{must_even_number, must_whole_number};
use opencv::core::Point2f;

#[derive(Debug)]
pub struct RailStationGenerator {
    pub start: Point2f,
    pub wagon_size: u32,
}

impl LuaCommandBatch for RailStationGenerator {
    fn make_lua_batch(self, lua_commands: &mut Vec<Box<dyn LuaCommand>>) {
        must_whole_number(self.start);
        must_even_number(self.start);

        self.make_pocket_rail_loop(lua_commands);
        self.make_station(lua_commands);
    }
}

impl RailStationGenerator {
    /// ---------------------
    ///                       -
    ///                         -
    ///                       -
    /// ---------------------
    fn make_pocket_rail_loop(&self, creation_commands: &mut Vec<Box<dyn LuaCommand>>) {
        let x_end = self.x_end();

        // Parallel lines
        for y in [self.start.y as i32, self.start.y as i32 + 22] {
            for x in (self.start.x as i32..x_end + 2).step_by(2) {
                creation_commands.push(
                    FacSurfaceCreateEntity::new_params(
                        "straight-rail",
                        Point2f {
                            // must be odd
                            x: x as f32 + 1.0,
                            y: y as f32 + 1.0,
                        },
                        CreateParam::direction(RailDirection::Left),
                    )
                    .into_boxed(),
                );
            }
        }

        // Curve up
        todo!("");
        // rail_degrees_east(Point2f {
        //     x: x_end as f32,
        //     y: self.start.y + 10.0,
        // })
        // .into_iter()
        // .for_each(|e| creation_commands.push(e));

        // Curve down
        todo!("");
        // rail_degrees_north(Point2f {
        //     x: x_end as f32 + 2.0,
        //     y: self.start.y,
        // })
        // .into_iter()
        // .for_each(|e| creation_commands.push(e));
    }

    /// ---------------------
    ///                     s -
    ///                         -
    ///                       -
    /// ---------------------
    fn make_station(&self, creation_commands: &mut Vec<Box<dyn LuaCommand>>) {
        let x_end = self.x_end();
        creation_commands.push(
            FacSurfaceCreateEntity::new_params(
                "train-stop",
                Point2f {
                    // must be odd
                    x: x_end as f32 + 1.0,
                    y: self.start.y + 3.0,
                },
                CreateParam::direction(RailDirection::Left),
            )
            .into_boxed(),
        );
    }

    fn x_end(&self) -> i32 {
        let wagons_to_rails = self.wagon_size * 6;
        let x_end = self.start.x as i32 + wagons_to_rails as i32;
        x_end
    }
}
