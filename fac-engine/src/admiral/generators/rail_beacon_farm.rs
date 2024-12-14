use crate::admiral::generators::beacon_farm::BeaconFarmGenerator;
use crate::admiral::generators::join_commands;
use crate::admiral::lua_command::LuaCommand;
use crate::admiral::lua_command::fac_surface_create_entity::{CreateParam, FacSurfaceCreateEntity};
use crate::common::cvpoint::Point2f;
use crate::game_entities::direction::RailDirection;
use tracing::info;

#[derive(Debug)]
pub struct RailBeaconFarmGenerator {
    pub inner: BeaconFarmGenerator,
}

impl LuaCommand for RailBeaconFarmGenerator {
    fn make_lua(&self) -> String {
        let mut creation_commands: Vec<Box<dyn LuaCommand>> = Vec::new();

        // creation_commands.push(Box::new(BeaconFarmGenerator {
        //     start: Point2f {
        //         x: self.inner.start.x + (self.inner.cell_size * 3) as f32,
        //         y: self.inner.start.y + (self.inner.cell_size * 3) as f32,
        //     },
        //     ..self.inner
        // }));
        //
        for y in [0, self.inner.height * (self.inner.cell_size) * 3] {
            for x in 0..(self.inner.width * self.inner.cell_size) {
                creation_commands.push(
                    FacSurfaceCreateEntity::new_params(
                        "straight-rail",
                        Point2f {
                            x: (self.inner.start.x + (x * 2) as f32).round(),
                            y: self.inner.start.y + y as f32,
                        },
                        CreateParam::direction(RailDirection::Left),
                    )
                    .into_boxed(),
                );
            }
        }

        todo!();
        // rail_degrees_south(self.inner.start)
        //     .into_iter()
        //     .for_each(|e| creation_commands.push(e));

        info!("creating {} elements", creation_commands.len());
        join_commands(creation_commands.iter())
    }
}