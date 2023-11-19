use crate::admiral::generators::beacon_farm::BeaconFarmGenerator;
use crate::admiral::lua_command::{
    direction_params, recipe_module_params_exact, recipe_params_exact, FacSurfaceCreateEntity,
    FacSurfaceCreateEntitySafe, LuaCommand, LuaCommandBatch, DEFAULT_SURFACE_VAR,
};
use num_format::Locale::lu;
use opencv::core::Point2f;
use std::collections::HashMap;
use tracing::debug;

#[derive(Debug)]
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
    fn make_assemblers(&self, lua_commands: &mut Vec<Box<dyn LuaCommand>>) {
        for x in 0..self.inner.width {
            for y in 0..self.inner.height {
                lua_commands.push(Box::new(FacSurfaceCreateEntitySafe {
                    inner: FacSurfaceCreateEntity {
                        name: "assembling-machine-3".to_string(),
                        params: recipe_params_exact("beacon"),
                        position: Point2f {
                            x: self.inner.start.x + (x as f32 * 9f32) + 3.0,
                            y: self.inner.start.y + (y as f32 * 9f32) + 3.0,
                        },
                        surface_var: DEFAULT_SURFACE_VAR.to_string(),
                        extra: vec![
                            "admiral_create.get_module_inventory().insert(\"speed-module-3\")"
                                .to_string(),
                        ],
                    },
                }));
            }
        }
    }

    fn make_output_chests(&self, lua_commands: &mut Vec<Box<dyn LuaCommand>>) {
        for x in 0..self.inner.width {
            for y in 0..self.inner.height {
                let mut total = 0.0;
                for num in &self.chests {
                    if let AssemblerChest::Output { is_purple } = num {
                        lua_commands.push(Box::new(FacSurfaceCreateEntitySafe {
                            inner: FacSurfaceCreateEntity {
                                name: "stack-inserter".to_string(),
                                params: direction_params("west"),
                                position: Point2f {
                                    x: self.inner.start.x + (x as f32 * 9f32) + 5.0,
                                    y: self.inner.start.y + (y as f32 * 9f32) + 2.0 + total,
                                },
                                surface_var: DEFAULT_SURFACE_VAR.to_string(),
                                extra: Vec::new(),
                            },
                        }));

                        let name = if *is_purple {
                            "logistic-chest-active-provider"
                        } else {
                            "logistic-chest-passive-provider"
                        }
                        .to_string();
                        lua_commands.push(Box::new(FacSurfaceCreateEntitySafe {
                            inner: FacSurfaceCreateEntity {
                                name,
                                params: HashMap::new(),
                                position: Point2f {
                                    x: self.inner.start.x + (x as f32 * 9f32) + 6.0,
                                    y: self.inner.start.y + (y as f32 * 9f32) + 2.0 + total,
                                },
                                surface_var: DEFAULT_SURFACE_VAR.to_string(),
                                extra: Vec::new(),
                            },
                        }));
                        total = total + 1.0;
                        debug!("insert {}", total)
                    }
                }
            }
        }
    }

    fn make_input_chests(&self, lua_commands: &mut Vec<Box<dyn LuaCommand>>) {
        for x in 0..self.inner.width {
            for y in 0..self.inner.height {
                let mut total = 0.0;
                for num in &self.chests {
                    if let AssemblerChest::Input { name, count } = num {
                        lua_commands.push(Box::new(FacSurfaceCreateEntitySafe {
                            inner: FacSurfaceCreateEntity {
                                name: "stack-inserter".to_string(),
                                params: direction_params("south"),
                                position: Point2f {
                                    x: self.inner.start.x + (x as f32 * 9f32) + 2.0 + total,
                                    y: self.inner.start.y + (y as f32 * 9f32) + 5.0,
                                },
                                surface_var: DEFAULT_SURFACE_VAR.to_string(),
                                extra: Vec::new(),
                            },
                        }));

                        lua_commands.push(Box::new(FacSurfaceCreateEntitySafe {
                            inner: FacSurfaceCreateEntity {
                                name: "logistic-chest-requester".to_string(),
                                params: HashMap::new(),
                                position: Point2f {
                                    x: self.inner.start.x + (x as f32 * 9f32) + 2.0 + total,
                                    y: self.inner.start.y + (y as f32 * 9f32) + 6.0,
                                },
                                surface_var: DEFAULT_SURFACE_VAR.to_string(),
                                extra: vec![
                                    "admiral_create.request_from_buffers = true".to_string(),
                                    format!("admiral_create.set_request_slot({{ name='{}', count={} }},1) ", name, count),
                                ],
                            },
                        }));
                        total = total + 1.0;
                        debug!("insert {}", total)
                    }
                }
            }
        }
    }

    fn make_buffer_chests(&self, lua_commands: &mut Vec<Box<dyn LuaCommand>>) {
        for x in 0..self.inner.width {
            for y in 0..self.inner.height {
                let mut total = 0.0;
                for num in &self.chests {
                    if let AssemblerChest::Buffer { name, count } = num {
                        lua_commands.push(Box::new(FacSurfaceCreateEntitySafe {
                            inner: FacSurfaceCreateEntity {
                                name: "logistic-chest-buffer".to_string(),
                                params: HashMap::new(),
                                position: Point2f {
                                    x: self.inner.start.x + (x as f32 * 9f32) + 2.0 + total,
                                    y: self.inner.start.y + (y as f32 * 9f32) + 7.0,
                                },
                                surface_var: DEFAULT_SURFACE_VAR.to_string(),
                                extra: vec![format!(
                                    "admiral_create.set_request_slot({{ name='{}', count={} }},1) ",
                                    name, count
                                )],
                            },
                        }));
                        total = total + 1.0;
                        debug!("insert {}", total)
                    }
                }
            }
        }
    }

    fn make_power(&self, lua_commands: &mut Vec<Box<dyn LuaCommand>>) {
        for x in 0..self.inner.width.div_ceil(2) {
            for y in 0..self.inner.height.div_ceil(2) {
                lua_commands.push(Box::new(FacSurfaceCreateEntitySafe {
                    inner: FacSurfaceCreateEntity {
                        name: "substation".to_string(),
                        params: HashMap::new(),
                        position: Point2f {
                            x: self.inner.start.x + (x as f32 * 18f32) + 6.5,
                            y: self.inner.start.y + (y as f32 * 18f32) + 6.5,
                        },
                        surface_var: DEFAULT_SURFACE_VAR.to_string(),
                        extra: Vec::new(),
                    },
                }));
            }
        }
    }

    fn make_power_interface(&self, lua_commands: &mut Vec<Box<dyn LuaCommand>>) {
        lua_commands.push(Box::new(FacSurfaceCreateEntitySafe {
            inner: FacSurfaceCreateEntity {
                name: "electric-energy-interface".to_string(),
                params: HashMap::new(),
                position: Point2f {
                    x: self.inner.start.x - 2.5,
                    y: self.inner.start.y - 0.5,
                },
                surface_var: DEFAULT_SURFACE_VAR.to_string(),
                extra: Vec::new(),
            },
        }));
    }
}
