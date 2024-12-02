use crate::admiral::generators::beacon_farm::BeaconFarmGenerator;
use crate::admiral::generators::{XyGridPosition, xy_grid};
use crate::admiral::lua_command::fac_surface_create_entity::{CreateParam, FacSurfaceCreateEntity};
use crate::admiral::lua_command::{LuaCommand, LuaCommandBatch};
use crate::common::cvpoint::Point2f;
use crate::game_entities::direction::RailDirection;

pub const ASSEMBLER_SIZE: u32 = 3;

#[derive(Debug, Clone)]
pub enum AssemblerChest {
    Input { name: String, count: u32 },
    Output { is_purple: bool },
    Buffer { name: String, count: u32 },
}

#[derive(Debug)]
pub struct AssemblerFarmGenerator {
    pub inner: BeaconFarmGenerator,
    pub chests: Vec<AssemblerChest>,
}

impl LuaCommandBatch for AssemblerFarmGenerator {
    fn make_lua_batch(self, lua_commands: &mut Vec<Box<dyn LuaCommand>>) {
        self.make_assemblers(lua_commands);
        self.make_input_chests(lua_commands);
        self.make_output_chests(lua_commands);
        self.make_buffer_chests(lua_commands);

        self.make_power(lua_commands);
        self.make_power_interface(lua_commands);
        self.inner.make_lua_batch(lua_commands);
    }
}

impl AssemblerFarmGenerator {
    fn assembler_xy_grid(&self) -> impl Iterator<Item = XyGridPosition> {
        xy_grid(
            self.inner.start.x as i32,
            self.inner.start.y as i32,
            self.inner.width,
            self.inner.height,
            (self.inner.cell_size - 1) * ASSEMBLER_SIZE,
        )
    }

    fn make_assemblers(&self, lua_commands: &mut Vec<Box<dyn LuaCommand>>) {
        for pos in self.assembler_xy_grid() {
            lua_commands.push(
                FacSurfaceCreateEntity::new_params_commands(
                    "assembling-machine-3",
                    Point2f {
                        x: pos.x as f32 + 5.5,
                        y: pos.y as f32 + 5.5,
                    },
                    CreateParam::recipe_str("steel-crate"),
                    vec![
                        "admiral_create.get_module_inventory().insert(\"speed-module-3\")"
                            .to_string(),
                    ],
                )
                .into_boxed(),
            );
        }
    }

    fn make_output_chests(&self, lua_commands: &mut Vec<Box<dyn LuaCommand>>) {
        for pos in self.assembler_xy_grid() {
            let mut total = 0.0;
            for chest in &self.chests {
                if let AssemblerChest::Output { is_purple } = chest {
                    let x = pos.x as f32 + 6.5 - total;

                    lua_commands.push(
                        FacSurfaceCreateEntity::new_params(
                            "assembling-machine-3",
                            Point2f {
                                x,
                                y: pos.y as f32 + 3.5,
                            },
                            CreateParam::direction(RailDirection::Down),
                        )
                        .into_boxed(),
                    );

                    lua_commands.push(
                        FacSurfaceCreateEntity::new(
                            if *is_purple {
                                "logistic-chest-active-provider"
                            } else {
                                "logistic-chest-passive-provider"
                            },
                            Point2f {
                                x,
                                y: pos.y as f32 + 2.5,
                            },
                        )
                        .into_boxed(),
                    );
                    total += 1.0;
                }
            }
        }
    }

    fn make_input_chests(&self, lua_commands: &mut Vec<Box<dyn LuaCommand>>) {
        for pos in self.assembler_xy_grid() {
            let mut total = 0.0;
            for chest in &self.chests {
                if let AssemblerChest::Input { name, count } = chest {
                    let y = pos.y as f32 + 6.5 - total;
                    lua_commands.push(
                        FacSurfaceCreateEntity::new_params(
                            "stack-inserter",
                            Point2f {
                                x: pos.x as f32 + 3.5,
                                y,
                            },
                            CreateParam::direction(RailDirection::Left),
                        )
                        .into_boxed(),
                    );

                    lua_commands.push(
                        FacSurfaceCreateEntity::new_commands(
                            "logistic-chest-requester",
                            Point2f {
                                x: pos.x as f32 + 2.5,
                                y,
                            },
                            vec![
                                "admiral_create.request_from_buffers = true".to_string(),
                                format!(
                                    "admiral_create.set_request_slot({{ name='{}', count={} }},1) ",
                                    name, count
                                ),
                            ],
                        )
                        .into_boxed(),
                    );
                    total += 1.0;
                }
            }
        }
    }

    fn make_buffer_chests(&self, lua_commands: &mut Vec<Box<dyn LuaCommand>>) {
        for pos in self.assembler_xy_grid() {
            let mut total = 0.0;
            for num in &self.chests {
                if let AssemblerChest::Buffer { name, count } = num {
                    lua_commands.push(
                        FacSurfaceCreateEntity::new_commands(
                            "logistic-chest-buffer",
                            Point2f {
                                x: pos.x as f32 + 2.5 + total,
                                y: pos.y as f32 + 7.5,
                            },
                            vec![format!(
                                "admiral_create.set_request_slot({{ name='{}', count={} }},1) ",
                                name, count
                            )],
                        )
                        .into_boxed(),
                    );
                    total += 1.0;
                }
            }
        }
    }

    fn make_power(&self, lua_commands: &mut Vec<Box<dyn LuaCommand>>) {
        for pos in self.assembler_xy_grid() {
            if pos.ix % 2 != 0 {
                continue;
            }
            lua_commands.push(
                FacSurfaceCreateEntity::new("substation", Point2f {
                    x: pos.x as f32 + 3.0,
                    y: pos.y as f32 + 3.0,
                })
                .into_boxed(),
            );

            lua_commands.push(
                FacSurfaceCreateEntity::new("small-lamp", Point2f {
                    x: pos.x as f32 + 7.5,
                    y: pos.y as f32 + 7.5,
                })
                .into_boxed(),
            );
        }
    }

    fn make_power_interface(&self, lua_commands: &mut Vec<Box<dyn LuaCommand>>) {
        lua_commands.push(
            FacSurfaceCreateEntity::new("electric-energy-interface", Point2f {
                x: self.inner.start.x - 2.5,
                y: self.inner.start.y - 0.5,
            })
            .into_boxed(),
        );
    }
}
