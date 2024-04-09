//! How Factorio does rail turning

use crate::admiral::lua_command::fac_surface_create_entity::{
    FacSurfaceCreateEntity, FactoDirection,
};
use crate::admiral::lua_command::LuaCommand;
use crate::navigator::mori::RailDirection;
use crate::surfacev::vpoint::VPoint;
use opencv::core::Point2f;

pub fn rail_degrees_north(start: Point2f) -> Vec<Box<dyn LuaCommand>> {
    vec![
        FacSurfaceCreateEntity::new_rail_curved_facto(
            Point2f {
                x: start.x + 10.0,
                y: start.y + 8.0,
            },
            FactoDirection::North,
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
                x: start.x + 4.0,
                y: start.y + 2.0,
            },
            FactoDirection::SouthEast,
        )
        .into_boxed(),
    ]
}

pub fn rail_degrees_south(start: Point2f) -> Vec<Box<dyn LuaCommand>> {
    vec![
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

pub fn rail_degrees_west(start: Point2f) -> Vec<Box<dyn LuaCommand>> {
    vec![
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

pub fn rail_degrees_east(start: Point2f) -> Vec<Box<dyn LuaCommand>> {
    vec![
        FacSurfaceCreateEntity::new_rail_curved_facto(
            Point2f {
                x: start.x + 4.0,
                y: start.y + 10.0,
            },
            FactoDirection::East,
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
                x: start.x + 10.0,
                y: start.y + 4.0,
            },
            FactoDirection::SouthWest,
        )
        .into_boxed(),
    ]
}

pub fn dual_rail_north(top_right_corner: VPoint, commands: &mut Vec<Box<dyn LuaCommand>>) {
    commands.append(&mut rail_degrees_north(
        top_right_corner.move_xy(4, 0).to_f32_with_offset(0.0),
    ));
    commands.append(&mut rail_degrees_north(
        top_right_corner.move_xy(0, 4).to_f32_with_offset(0.0),
    ));

    // outer top
    add_rail(top_right_corner, RailDirection::Left, commands);
    add_rail(
        top_right_corner.move_xy(2, 0),
        RailDirection::Left,
        commands,
    );

    // outer bottom
    add_rail(
        top_right_corner.move_xy(14, 12),
        RailDirection::Down,
        commands,
    );
    add_rail(
        top_right_corner.move_xy(14, 14),
        RailDirection::Down,
        commands,
    );
}

pub fn dual_rail_south(top_right_corner: VPoint, commands: &mut Vec<Box<dyn LuaCommand>>) {
    commands.append(&mut rail_degrees_south(
        top_right_corner.move_xy(4, 0).to_f32_with_offset(0.0),
    ));
    commands.append(&mut rail_degrees_south(
        top_right_corner.move_xy(0, 4).to_f32_with_offset(0.0),
    ));

    // outer top
    add_rail(top_right_corner, RailDirection::Down, commands);
    add_rail(
        top_right_corner.move_xy(0, 2),
        RailDirection::Down,
        commands,
    );

    // outer bottom
    add_rail(
        top_right_corner.move_xy(12, 14),
        RailDirection::Left,
        commands,
    );
    add_rail(
        top_right_corner.move_xy(14, 14),
        RailDirection::Left,
        commands,
    );
}

pub fn dual_rail_east(top_right_corner: VPoint, commands: &mut Vec<Box<dyn LuaCommand>>) {
    commands.append(&mut rail_degrees_east(
        top_right_corner.move_xy(4, 4).to_f32_with_offset(0.0),
    ));
    commands.append(&mut rail_degrees_east(
        top_right_corner.move_xy(0, 0).to_f32_with_offset(0.0),
    ));

    // outer top
    add_rail(
        top_right_corner.move_xy(14, 0),
        RailDirection::Down,
        commands,
    );
    add_rail(
        top_right_corner.move_xy(14, 2),
        RailDirection::Down,
        commands,
    );

    // outer bottom
    add_rail(
        top_right_corner.move_xy(0, 14),
        RailDirection::Left,
        commands,
    );
    add_rail(
        top_right_corner.move_xy(2, 14),
        RailDirection::Left,
        commands,
    );
}

pub fn dual_rail_west(top_right_corner: VPoint, commands: &mut Vec<Box<dyn LuaCommand>>) {
    commands.append(&mut rail_degrees_west(
        top_right_corner.move_xy(4, 4).to_f32_with_offset(0.0),
    ));
    commands.append(&mut rail_degrees_west(
        top_right_corner.move_xy(0, 0).to_f32_with_offset(0.0),
    ));

    // outer top
    add_rail(
        top_right_corner.move_xy(14, 0),
        RailDirection::Left,
        commands,
    );
    add_rail(
        top_right_corner.move_xy(12, 0),
        RailDirection::Left,
        commands,
    );

    // outer bottom
    add_rail(
        top_right_corner.move_xy(0, 12),
        RailDirection::Down,
        commands,
    );
    add_rail(
        top_right_corner.move_xy(0, 14),
        RailDirection::Down,
        commands,
    );
}

fn add_rail(point: VPoint, direction: RailDirection, commands: &mut Vec<Box<dyn LuaCommand>>) {
    commands.push(
        FacSurfaceCreateEntity::new_rail_straight(point.to_f32_with_offset(0.0), direction)
            .into_boxed(),
    );
}
