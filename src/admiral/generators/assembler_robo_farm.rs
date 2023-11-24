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
const ROBOPORT_BLOCK_SIZE: u32 = ROBOPORT_SIZE * ROBO_POLE_STEP;

const ASSEMBLER_CELL_COUNT: u32 = 4;

#[derive(Debug)]
pub struct AssemblerRoboFarmGenerator {
    pub start: Point2f,
    pub row_count: u32,
    pub robo_height: u32,
    pub assembler_width: u32,
    pub assembler_height: u32,
    pub chests: Vec<AssemblerChest>,
}

impl LuaCommandBatch for AssemblerRoboFarmGenerator {
    fn make_lua_batch(self, lua_commands: &mut Vec<Box<dyn LuaCommand>>) {
        let width = 5;
        let height = 5;
        trace!("div {} div {}", 5_u32.div_ceil(2), 3_u32.div_ceil(2));

        let row_assembly_width = ((self.assembler_width - 1) * BEACON_SIZE * ASSEMBLER_CELL_COUNT);
        let row_assembly_height = ((self.assembler_height - 1) * BEACON_SIZE * ASSEMBLER_CELL_COUNT)
            as f32
            + BEACON_SIZE as f32;

        let row_robo_width_count = row_assembly_width.div_floor(ROBOPORT_BLOCK_SIZE);
        let row_robo_height_count = self.robo_height;
        let row_robo_height = (row_robo_height_count * ROBOPORT_BLOCK_SIZE) as f32; //+ (BEACON_SIZE * 2) as f32;

        for row in 0..=self.row_count {
            let row_height = (row as f32 * (row_robo_height + row_assembly_height))
                + BEACON_SIZE as f32
                + BEACON_SIZE as f32;
            debug!("row_height {}", row_height);

            make_robo_square(
                self.start.x as i32,
                (self.start.y + row_height + 1.0) as i32,
                row_robo_width_count,
                row_robo_height_count,
                lua_commands,
            );

            let assembler_start_y = self.start.y + row_height + row_robo_height + 0.5;
            if row != self.row_count {
                AssemblerFarmGenerator {
                    inner: BeaconFarmGenerator {
                        width: self.assembler_width,
                        height: self.assembler_height,
                        start: Point2f {
                            x: self.start.x - 0.5,
                            y: assembler_start_y,
                        },
                        cell_size: ASSEMBLER_CELL_COUNT,
                        module: "speed-module-3".to_string(),
                    },
                    chests: self.chests.clone(),
                }
                .make_lua_batch(lua_commands);

                // big pole connects top of assembly to robo's big pole
                lua_commands.push(Box::new(FacSurfaceCreateEntitySafe {
                    inner: FacSurfaceCreateEntity {
                        name: "big-electric-pole".to_string(),
                        params: HashMap::new(),
                        position: Point2f {
                            x: self.start.x - 5.0 + row_height,
                            y: assembler_start_y,
                        },
                        surface_var: DEFAULT_SURFACE_VAR.to_string(),
                        extra: Vec::new(),
                    },
                }));
            }
        }
    }
}

fn make_robo_square(
    start_x: i32,
    start_y: i32,
    width: u32,
    height: u32,
    lua_commands: &mut Vec<Box<dyn LuaCommand>>,
) {
    for block_pos in xy_grid(start_x, start_y, width, height, ROBOPORT_BLOCK_SIZE) {
        for pos in xy_grid(block_pos.x, block_pos.y, 5, 5, ROBOPORT_SIZE) {
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
}
