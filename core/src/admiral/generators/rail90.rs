//! How Factorio does rail turning

use crate::admiral::lua_command::fac_surface_create_entity::{
    FacSurfaceCreateEntity, FactoDirection,
};
use crate::admiral::lua_command::LuaCommand;
use crate::navigator::mori::RailDirection;
use crate::surfacev::bit_grid::BitGrid;
use crate::surfacev::vpoint::VPoint;
use lazy_static::lazy_static;
use opencv::core::Point2f;
use std::cell::LazyCell;

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
    top_right_corner.assert_even_position();

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

pub fn dual_rail_north_empty() -> BitGrid {
    const DUAL_RAIL_NORTH_EMPTY_SPACES_CONFIG: [u64; 4] = [
        0xff003ffe1fff0f,
        0xfc703e3e1f1f0f1,
        0xfc78fe3cff1cff1c,
        0xff8cffccffccffcc,
    ];
    BitGrid::from_u64(DUAL_RAIL_NORTH_EMPTY_SPACES_CONFIG)
}

pub fn dual_rail_south(top_right_corner: VPoint, commands: &mut Vec<Box<dyn LuaCommand>>) {
    top_right_corner.assert_even_position();

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

pub fn dual_rail_south_empty() -> BitGrid {
    const DUAL_RAIL_SOUTH_EMPTY_SPACES_CONFIG: [u64; 4] = [
        0x33ff33ff33ff31ff,
        0x38ff38ff3c7f1e3f,
        0x8f0f8f87c7c0e3f0,
        0xf0fff87ffc00ff00,
    ];

    BitGrid::from_u64(DUAL_RAIL_SOUTH_EMPTY_SPACES_CONFIG)
}

pub fn dual_rail_east(top_right_corner: VPoint, commands: &mut Vec<Box<dyn LuaCommand>>) {
    top_right_corner.assert_even_position();

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

pub fn dual_rail_east_empty() -> BitGrid {
    const DUAL_RAIL_EAST_EMPTY_SPACES_CONFIG: [u64; 4] = [
        0xffccffccffccff8c,
        0xff1cff1cfe3cfc78,
        0xf0f1e1f103e30fc7,
        0xff0ffe1f003f00ff,
    ];
    BitGrid::from_u64(DUAL_RAIL_EAST_EMPTY_SPACES_CONFIG)
}

pub fn dual_rail_west(top_right_corner: VPoint, commands: &mut Vec<Box<dyn LuaCommand>>) {
    top_right_corner.assert_even_position();

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

pub fn dual_rail_west_empty() -> BitGrid {
    const DUAL_RAIL_WEST_EMPTY_SPACES_CONFIG: [u64; 4] = [
        0xff00fc00f87ff0ff,
        0xe3f0c7c08f878f0f,
        0x1e3f3c7f38ff38ff,
        0x31ff33ff33ff33ff,
    ];
    BitGrid::from_u64(DUAL_RAIL_WEST_EMPTY_SPACES_CONFIG)
}

fn add_rail(point: VPoint, direction: RailDirection, commands: &mut Vec<Box<dyn LuaCommand>>) {
    commands.push(
        FacSurfaceCreateEntity::new_rail_straight(point.to_f32_with_offset(1.0), direction)
            .into_boxed(),
    );
}
