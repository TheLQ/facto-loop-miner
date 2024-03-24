use crate::admiral::lua_command::fac_surface_create_entity::FacSurfaceCreateEntity;
use crate::admiral::lua_command::fac_surface_create_entity_safe::FacSurfaceCreateEntitySafe;
use crate::admiral::lua_command::{direction_params_exact, LuaCommand, DEFAULT_SURFACE_VAR};
use opencv::core::Point2f;
use std::collections::HashMap;

pub fn rail_degrees_90(start: Point2f) -> [Box<dyn LuaCommand>; 5] {
    [
        Box::new(FacSurfaceCreateEntitySafe {
            inner: FacSurfaceCreateEntity {
                name: "straight-rail".to_string(),
                params: HashMap::new(),
                position: Point2f {
                    x: start.x + 1.0,
                    y: start.y + 1.0,
                },
                surface_var: DEFAULT_SURFACE_VAR.to_string(),
                extra: Vec::new(),
            },
        }),
        Box::new(FacSurfaceCreateEntitySafe {
            inner: FacSurfaceCreateEntity {
                name: "curved-rail".to_string(),
                params: direction_params_exact("4"),
                position: Point2f {
                    x: start.x + 2.0,
                    y: start.y + 6.0,
                },
                surface_var: DEFAULT_SURFACE_VAR.to_string(),
                extra: Vec::new(),
            },
        }),
        Box::new(FacSurfaceCreateEntitySafe {
            inner: FacSurfaceCreateEntity {
                name: "straight-rail".to_string(),
                params: direction_params_exact("5"),
                position: Point2f {
                    x: start.x + 5.0,
                    y: start.y + 9.0,
                },
                surface_var: DEFAULT_SURFACE_VAR.to_string(),
                extra: Vec::new(),
            },
        }),
        Box::new(FacSurfaceCreateEntitySafe {
            inner: FacSurfaceCreateEntity {
                name: "curved-rail".to_string(),
                params: direction_params_exact("7"),
                position: Point2f {
                    x: start.x + 8.0,
                    y: start.y + 12.0,
                },
                surface_var: DEFAULT_SURFACE_VAR.to_string(),
                extra: Vec::new(),
            },
        }),
        Box::new(FacSurfaceCreateEntitySafe {
            inner: FacSurfaceCreateEntity {
                name: "straight-rail".to_string(),
                params: direction_params_exact("2"),
                position: Point2f {
                    x: start.x + 13.0,
                    y: start.y + 13.0,
                },
                surface_var: DEFAULT_SURFACE_VAR.to_string(),
                extra: Vec::new(),
            },
        }),
    ]
}

pub fn rail_degrees_180(start: Point2f) -> [Box<dyn LuaCommand>; 5] {
    [
        Box::new(FacSurfaceCreateEntitySafe {
            inner: FacSurfaceCreateEntity {
                name: "curved-rail".to_string(),
                params: direction_params_exact("6"),
                position: Point2f {
                    x: start.x + 8.0,
                    y: start.y + 2.0,
                },
                surface_var: DEFAULT_SURFACE_VAR.to_string(),
                extra: Vec::new(),
            },
        }),
        Box::new(FacSurfaceCreateEntitySafe {
            inner: FacSurfaceCreateEntity {
                name: "straight-rail".to_string(),
                params: direction_params_exact("2"),
                position: Point2f {
                    x: start.x + 13.0,
                    y: start.y + 1.0,
                },
                surface_var: DEFAULT_SURFACE_VAR.to_string(),
                extra: Vec::new(),
            },
        }),
        Box::new(FacSurfaceCreateEntitySafe {
            inner: FacSurfaceCreateEntity {
                name: "straight-rail".to_string(),
                params: direction_params_exact("7"),
                position: Point2f {
                    x: start.x + 5.0,
                    y: start.y + 5.0,
                },
                surface_var: DEFAULT_SURFACE_VAR.to_string(),
                extra: Vec::new(),
            },
        }),
        Box::new(FacSurfaceCreateEntitySafe {
            inner: FacSurfaceCreateEntity {
                name: "curved-rail".to_string(),
                params: direction_params_exact("1"),
                position: Point2f {
                    x: start.x + 2.0,
                    y: start.y + 8.0,
                },
                surface_var: DEFAULT_SURFACE_VAR.to_string(),
                extra: Vec::new(),
            },
        }),
        Box::new(FacSurfaceCreateEntitySafe {
            inner: FacSurfaceCreateEntity {
                name: "straight-rail".to_string(),
                params: HashMap::new(),
                position: Point2f {
                    x: start.x + 1.0,
                    y: start.y + 13.0,
                },
                surface_var: DEFAULT_SURFACE_VAR.to_string(),
                extra: Vec::new(),
            },
        }),
    ]
}

pub fn rail_degrees_270(start: Point2f) -> [Box<dyn LuaCommand>; 3] {
    [
        Box::new(FacSurfaceCreateEntitySafe {
            inner: FacSurfaceCreateEntity {
                name: "curved-rail".to_string(),
                params: direction_params_exact("3"),
                position: Point2f {
                    x: start.x + 4.0,
                    y: start.y + 2.0,
                },
                surface_var: DEFAULT_SURFACE_VAR.to_string(),
                extra: Vec::new(),
            },
        }),
        Box::new(FacSurfaceCreateEntitySafe {
            inner: FacSurfaceCreateEntity {
                name: "straight-rail".to_string(),
                params: direction_params_exact("1"),
                position: Point2f {
                    x: start.x + 7.0,
                    y: start.y + 5.0,
                },
                surface_var: DEFAULT_SURFACE_VAR.to_string(),
                extra: Vec::new(),
            },
        }),
        Box::new(FacSurfaceCreateEntitySafe {
            inner: FacSurfaceCreateEntity {
                name: "curved-rail".to_string(),
                params: HashMap::new(),
                position: Point2f {
                    x: start.x + 10.0,
                    y: start.y + 8.0,
                },
                surface_var: DEFAULT_SURFACE_VAR.to_string(),
                extra: Vec::new(),
            },
        }),
    ]
}

pub fn rail_degrees_360(start: Point2f) -> [Box<dyn LuaCommand>; 3] {
    [
        // Box::new(FacSurfaceCreateEntitySafe {
        //     inner: FacSurfaceCreateEntity {
        //         name: "straight-rail".to_string(),
        //         params: HashMap::new(),
        //         position: Point2f {
        //             x: start.x + 13.0,
        //             y: start.y + 1.0,
        //         },
        //         surface_var: DEFAULT_SURFACE_VAR.to_string(), extra: Vec::new()
        //     },
        // }),
        Box::new(FacSurfaceCreateEntitySafe {
            inner: FacSurfaceCreateEntity {
                name: "curved-rail".to_string(),
                params: direction_params_exact("5"),
                position: Point2f {
                    x: start.x + 12.0,
                    y: start.y + 6.0,
                },
                surface_var: DEFAULT_SURFACE_VAR.to_string(),
                extra: Vec::new(),
            },
        }),
        Box::new(FacSurfaceCreateEntitySafe {
            inner: FacSurfaceCreateEntity {
                name: "curved-rail".to_string(),
                params: direction_params_exact("2"),
                position: Point2f {
                    x: start.x + 6.0,
                    y: start.y + 12.0,
                },
                surface_var: DEFAULT_SURFACE_VAR.to_string(),
                extra: Vec::new(),
            },
        }),
        Box::new(FacSurfaceCreateEntitySafe {
            inner: FacSurfaceCreateEntity {
                name: "straight-rail".to_string(),
                params: direction_params_exact("3"),
                position: Point2f {
                    x: start.x + 9.0,
                    y: start.y + 9.0,
                },
                surface_var: DEFAULT_SURFACE_VAR.to_string(),
                extra: Vec::new(),
            },
        }),
        // Box::new(FacSurfaceCreateEntitySafe {
        //     inner: FacSurfaceCreateEntity {
        //         name: "straight-rail".to_string(),
        //         params: direction_params_exact("2"),
        //         position: Point2f {
        //             x: start.x + 1.0,
        //             y: start.y + 13.0,
        //         },
        //         surface_var: DEFAULT_SURFACE_VAR.to_string(), extra: Vec::new()
        //     },
        // }),
    ]
}
