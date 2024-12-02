use crate::admiral::lua_command::fac_surface_create_entity::{CreateParam, FacSurfaceCreateEntity};
use crate::admiral::lua_command::{LuaCommand, LuaCommandBatch};
use crate::navigator::mori::RailDirection;
use crate::point::Point2f;

#[derive(Debug)]
pub struct RailStationPathfoundGenerator {
    pub start: Point2f,
    pub station: Point2f,
    pub pan: Point2f,
}

impl LuaCommandBatch for RailStationPathfoundGenerator {
    fn make_lua_batch(self, lua_commands: &mut Vec<Box<dyn LuaCommand>>) {
        lua_commands.push(
            FacSurfaceCreateEntity::new_params(
                "straight-rail",
                Point2f {
                    x: self.start.x,
                    y: self.start.y,
                },
                CreateParam::direction(RailDirection::Left),
            )
            .into_boxed(),
        );

        lua_commands.push(
            FacSurfaceCreateEntity::new_params(
                "straight-rail",
                Point2f {
                    x: self.station.x,
                    y: self.station.y,
                },
                CreateParam::direction(RailDirection::Left),
            )
            .into_boxed(),
        );

        lua_commands.push(
            FacSurfaceCreateEntity::new_params(
                "straight-rail",
                Point2f {
                    x: self.pan.x,
                    y: self.pan.y,
                },
                CreateParam::direction(RailDirection::Left),
            )
            .into_boxed(),
        );
    }
}
