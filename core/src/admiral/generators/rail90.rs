use crate::admiral::lua_command::fac_surface_create_entity::{
    FacSurfaceCreateEntity, FactoDirection,
};
use crate::admiral::lua_command::LuaCommand;
use opencv::core::Point2f;

pub fn rail_degrees_90(start: Point2f) -> [Box<dyn LuaCommand>; 3] {
    [
        FacSurfaceCreateEntity::new_rail_curved_facto(
            Point2f {
                x: start.x + 2.0,
                y: start.y + 4.0,
            },
            FactoDirection::South,
        )
        .into_boxed(),
        FacSurfaceCreateEntity::new_rail_straight_facto(
            Point2f {
                x: start.x + 5.0,
                y: start.y + 7.0,
            },
            FactoDirection::SouthWest,
        )
        .into_boxed(),
        FacSurfaceCreateEntity::new_rail_curved_facto(
            Point2f {
                x: start.x + 8.0,
                y: start.y + 10.0,
            },
            FactoDirection::NorthWest,
        )
        .into_boxed(),
    ]
}

pub fn rail_degrees_180(start: Point2f) -> [Box<dyn LuaCommand>; 3] {
    [
        FacSurfaceCreateEntity::new_rail_curved_facto(
            Point2f {
                x: start.x + 8.0,
                y: start.y + 2.0,
            },
            FactoDirection::West,
        )
        .into_boxed(),
        FacSurfaceCreateEntity::new_rail_straight_facto(
            Point2f {
                x: start.x + 5.0,
                y: start.y + 5.0,
            },
            FactoDirection::NorthWest,
        )
        .into_boxed(),
        FacSurfaceCreateEntity::new_rail_curved_facto(
            Point2f {
                x: start.x + 2.0,
                y: start.y + 8.0,
            },
            FactoDirection::NorthEast, // wtf?
        )
        .into_boxed(),
    ]
}

pub fn rail_degrees_270(start: Point2f) -> [Box<dyn LuaCommand>; 3] {
    [
        FacSurfaceCreateEntity::new_rail_curved_facto(
            Point2f {
                x: start.x + 4.0,
                y: start.y + 2.0,
            },
            FactoDirection::SouthEast,
        )
        .into_boxed(),
        FacSurfaceCreateEntity::new_rail_straight_facto(
            Point2f {
                x: start.x + 7.0,
                y: start.y + 5.0,
            },
            FactoDirection::NorthEast,
        )
        .into_boxed(),
        FacSurfaceCreateEntity::new_rail_curved_facto(
            Point2f {
                x: start.x + 10.0,
                y: start.y + 8.0,
            },
            FactoDirection::North,
        )
        .into_boxed(),
    ]
}

pub fn rail_degrees_360(start: Point2f) -> [Box<dyn LuaCommand>; 3] {
    [
        FacSurfaceCreateEntity::new_rail_curved_facto(
            Point2f {
                x: start.x + 10.0,
                y: start.y + 4.0,
            },
            FactoDirection::SouthWest,
        )
        .into_boxed(),
        FacSurfaceCreateEntity::new_rail_straight_facto(
            Point2f {
                x: start.x + 7.0,
                y: start.y + 7.0,
            },
            FactoDirection::SouthEast,
        )
        .into_boxed(),
        FacSurfaceCreateEntity::new_rail_curved_facto(
            Point2f {
                x: start.x + 4.0,
                y: start.y + 10.0,
            },
            FactoDirection::East,
        )
        .into_boxed(),
    ]
}
