use crate::admiral::generators::beacon_farm::BeaconFarmGenerator;
use crate::admiral::lua_command::{
    recipe_module_params_exact, recipe_params_exact, FacSurfaceCreateEntity,
    FacSurfaceCreateEntitySafe, LuaCommand, LuaCommandBatch, DEFAULT_SURFACE_VAR,
};
use num_format::Locale::lu;
use opencv::core::Point2f;
use std::collections::HashMap;

#[derive(Debug)]
pub struct AssemblerFarmGenerator {
    pub(crate) inner: BeaconFarmGenerator,
}

impl LuaCommandBatch for AssemblerFarmGenerator {
    fn make_lua_batch(self, lua_commands: &mut Vec<Box<dyn LuaCommand>>) {
        self.make_assemblers(lua_commands);
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
