use crate::admiral::lua_command::fac_surface_create_entity::FacSurfaceCreateEntity;
use crate::admiral::lua_command::LuaCommand;
use opencv::core::Point2f;

pub fn rail_degrees_90(start: Point2f) -> [Box<dyn LuaCommand>; 5] {
    [
        FacSurfaceCreateEntity::new_rail_straight_direction_num(
            Point2f {
                x: start.x + 1.0,
                y: start.y + 1.0,
            },
            4,
        )
        .into_boxed(),
        FacSurfaceCreateEntity::new_rail_curved_direction_num(
            Point2f {
                x: start.x + 2.0,
                y: start.y + 6.0,
            },
            4,
        )
        .into_boxed(),
        FacSurfaceCreateEntity::new_rail_straight_direction_num(
            Point2f {
                x: start.x + 5.0,
                y: start.y + 9.0,
            },
            5,
        )
        .into_boxed(),
        FacSurfaceCreateEntity::new_rail_curved_direction_num(
            Point2f {
                x: start.x + 8.0,
                y: start.y + 12.0,
            },
            7,
        )
        .into_boxed(),
        FacSurfaceCreateEntity::new_rail_straight_direction_num(
            Point2f {
                x: start.x + 13.0,
                y: start.y + 13.0,
            },
            2,
        )
        .into_boxed(),
    ]
}

pub fn rail_degrees_180(start: Point2f) -> [Box<dyn LuaCommand>; 5] {
    [
        FacSurfaceCreateEntity::new_rail_straight_direction_num(
            Point2f {
                x: start.x + 8.0,
                y: start.y + 2.0,
            },
            6,
        )
        .into_boxed(),
        FacSurfaceCreateEntity::new_rail_curved_direction_num(
            Point2f {
                x: start.x + 13.0,
                y: start.y + 1.0,
            },
            2,
        )
        .into_boxed(),
        FacSurfaceCreateEntity::new_rail_straight_direction_num(
            Point2f {
                x: start.x + 5.0,
                y: start.y + 5.0,
            },
            7,
        )
        .into_boxed(),
        FacSurfaceCreateEntity::new_rail_curved_direction_num(
            Point2f {
                x: start.x + 2.0,
                y: start.y + 8.0,
            },
            1,
        )
        .into_boxed(),
        FacSurfaceCreateEntity::new_rail_straight_direction_num(
            Point2f {
                x: start.x + 1.0,
                y: start.y + 13.0,
            },
            1,
        )
        .into_boxed(),
    ]
}

pub fn rail_degrees_270(start: Point2f) -> [Box<dyn LuaCommand>; 5] {
    todo!();
    [
        // todo not here before
        FacSurfaceCreateEntity::new_rail_straight_direction_num(
            Point2f {
                x: start.x + 4.0,
                y: start.y + 2.0,
            },
            6, // todo
        )
        .into_boxed(),
        FacSurfaceCreateEntity::new_rail_curved_direction_num(
            Point2f {
                x: start.x + 4.0,
                y: start.y + 2.0,
            },
            3,
        )
        .into_boxed(),
        FacSurfaceCreateEntity::new_rail_straight_direction_num(
            Point2f {
                x: start.x + 7.0,
                y: start.y + 5.0,
            },
            1,
        )
        .into_boxed(),
        FacSurfaceCreateEntity::new_rail_curved_direction_num(
            Point2f {
                x: start.x + 10.0,
                y: start.y + 8.0,
            },
            1, // todo
        )
        .into_boxed(),
        // todo not here before
        FacSurfaceCreateEntity::new_rail_straight_direction_num(
            Point2f {
                x: start.x + 10.0,
                y: start.y + 8.0,
            },
            1, // todo
        )
        .into_boxed(),
    ]
}

pub fn rail_degrees_360(start: Point2f) -> [Box<dyn LuaCommand>; 3] {
    todo!()
    /*
    [
        // Box::new(FacSurfaceCreateEntitySafe {
        //     inner: FacSurfaceCreateEntity {
        //         name: "straight-rail".to_string(),
        //         params: HashMap::new(),
        //         position: Point2f {
        //             x: start.x + 13.0,
        //             y: start.y + 1.0,
        //         },
        //          extra: Vec::new()
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
        //          extra: Vec::new()
        //     },
        // }),
    ]
    */
}
