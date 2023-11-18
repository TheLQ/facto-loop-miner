use crate::admiral::generators::beacon_farm::BeaconFarmGenerator;
use crate::admiral::generators::join_commands;
use crate::admiral::generators::rail90::{
    rail_degrees_180, rail_degrees_270, rail_degrees_360, rail_degrees_90,
};
use crate::admiral::lua_command::{
    direction_params, FacSurfaceCreateEntity, FacSurfaceCreateEntitySafe, LuaCommand,
    DEFAULT_SURFACE_VAR,
};
use opencv::core::Point2f;
use std::collections::HashMap;
use tracing::info;

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
                creation_commands.push(Box::new(FacSurfaceCreateEntitySafe {
                    inner: FacSurfaceCreateEntity {
                        name: "straight-rail".to_string(),
                        position: Point2f {
                            x: (self.inner.start.x + (x * 2) as f32).round(),
                            y: self.inner.start.y + y as f32,
                        },
                        surface_var: DEFAULT_SURFACE_VAR.to_string(),
                        params: direction_params("east"),
                    },
                }));
            }
        }

        rail_degrees_90(self.inner.start)
            .into_iter()
            .for_each(|e| creation_commands.push(e));

        info!("creating {} elements", creation_commands.len());
        join_commands(creation_commands.iter())
    }
}
