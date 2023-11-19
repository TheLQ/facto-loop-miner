use crate::admiral::generators::assembler_farm::{
    AssemblerChest, AssemblerFarmGenerator, ASSEMBLER_SIZE,
};
use crate::admiral::generators::beacon_farm::{BeaconFarmGenerator, BEACON_SIZE};
use crate::admiral::generators::xy_grid;
use crate::admiral::lua_command::{
    recipe_params_exact, FacSurfaceCreateEntity, FacSurfaceCreateEntitySafe, LuaCommand,
    LuaCommandBatch, DEFAULT_SURFACE_VAR,
};
use num_format::Locale::lu;
use opencv::core::{Point, Point2f};
use std::collections::HashMap;
use tracing::{debug, trace};

const ROBOPORT_SIZE: u32 = 4;

const ROBO_POLE_STEP: u32 = 5;
const ROBO_POLE_STEP_MIDDLE: u32 = ROBO_POLE_STEP.div_ceil(2);

#[derive(Debug)]
pub struct AssemblerRoboFarmGenerator {
    pub start: Point,
    pub column_count: u32,
    pub robo_width: u32,
    pub assembler_width: u32,
    pub assembler_height: u32,
    pub chests: Vec<AssemblerChest>,
}

impl LuaCommandBatch for AssemblerRoboFarmGenerator {
    fn make_lua_batch(self, lua_commands: &mut Vec<Box<dyn LuaCommand>>) {
        let width = 5;
        let height = 5;
        trace!("div {} div {}", 5_u32.div_ceil(2), 3_u32.div_ceil(2));

        let column_robo_width = self.robo_width * ROBOPORT_SIZE;
        let mut column_robo_height_count = ((self.assembler_height - 1) * BEACON_SIZE * 4);
        column_robo_height_count =
            column_robo_height_count - (column_robo_height_count % ROBOPORT_SIZE);
        column_robo_height_count = column_robo_height_count / ROBOPORT_SIZE;
        let column_assembly_width = ((self.assembler_width - 1) * BEACON_SIZE * 4);
        for column in 0..self.column_count {
            let column_size: i32 = (column * (column_robo_width + column_assembly_width)) as i32;
            debug!("column_size {}", column_size);
            make_robo_square(
                self.start.x + column_size,
                self.start.y,
                self.robo_width,
                column_robo_height_count,
                lua_commands,
            );
            AssemblerFarmGenerator {
                inner: BeaconFarmGenerator {
                    width: self.assembler_width,
                    height: self.assembler_height,
                    start: Point2f {
                        x: self.start.x as f32 + (ROBOPORT_SIZE * 5) as f32 - 0.5
                            + column_size as f32,
                        y: self.start.y as f32 + 0.5,
                    },
                    cell_size: 4,
                    module: "speed-module-3".to_string(),
                },
                chests: self.chests.clone(),
            }
            .make_lua_batch(lua_commands);
        }

        // self.inner.make_lua_batch(lua_commands);
    }
}

fn make_robo_square(
    start_x: i32,
    start_y: i32,
    width: u32,
    height: u32,
    lua_commands: &mut Vec<Box<dyn LuaCommand>>,
) {
    for pos in xy_grid(start_x, start_y, width, height, ROBOPORT_SIZE) {
        // debug!(
        //     "step_width {} step_height {} needle {}",
        //     pos.ix % ROBO_POLE_STEP,
        //     pos.iy % ROBO_POLE_STEP,
        //     ROBO_POLE_STEP_MIDDLE
        // );
        if (pos.ix % ROBO_POLE_STEP) + 1 == ROBO_POLE_STEP_MIDDLE
            && (pos.iy % ROBO_POLE_STEP) + 1 == ROBO_POLE_STEP_MIDDLE
        {
            lua_commands.push(Box::new(FacSurfaceCreateEntitySafe {
                inner: FacSurfaceCreateEntity {
                    name: "substation".to_string(),
                    params: HashMap::new(),
                    position: Point2f {
                        x: pos.x as f32 - 1.0,
                        y: pos.y as f32,
                    },
                    surface_var: DEFAULT_SURFACE_VAR.to_string(),
                    extra: Vec::new(),
                },
            }));
            lua_commands.push(Box::new(FacSurfaceCreateEntitySafe {
                inner: FacSurfaceCreateEntity {
                    name: "big-electric-pole".to_string(),
                    params: HashMap::new(),
                    position: Point2f {
                        x: pos.x as f32 + 1.0,
                        y: pos.y as f32,
                    },
                    surface_var: DEFAULT_SURFACE_VAR.to_string(),
                    extra: Vec::new(),
                },
            }));

            lua_commands.push(Box::new(FacSurfaceCreateEntitySafe {
                inner: FacSurfaceCreateEntity {
                    name: "small-lamp".to_string(),
                    params: HashMap::new(),
                    position: Point2f {
                        x: pos.x as f32 - 1.5,
                        y: pos.y as f32 + 1.5,
                    },
                    surface_var: DEFAULT_SURFACE_VAR.to_string(),
                    extra: Vec::new(),
                },
            }));
        } else {
            lua_commands.push(Box::new(FacSurfaceCreateEntitySafe {
                inner: FacSurfaceCreateEntity {
                    name: "roboport".to_string(),
                    params: recipe_params_exact("beacon"),
                    position: Point2f {
                        x: pos.x as f32,
                        y: pos.y as f32,
                    },
                    surface_var: DEFAULT_SURFACE_VAR.to_string(),
                    extra: Vec::new(),
                },
            }));
        }
    }
}
