//! How Factorio does rail turning

use crate::admiral::lua_command::fac_surface_create_entity::{
    FacSurfaceCreateEntity, FactoDirection,
};
use crate::admiral::lua_command::LuaCommand;
use crate::navigator::mori::RailDirection;
use crate::surfacev::bit_grid::{StaticBitGrid, GRID_16X16_SIZE};
use crate::surfacev::vpoint::VPoint;

pub fn rail_degrees_north(start: VPoint) -> Vec<Box<dyn LuaCommand>> {
    start.assert_odd_position();
    vec![
        FacSurfaceCreateEntity::new_rail_curved_facto(
            start.move_xy(9, 7).to_f32(),
            FactoDirection::North,
        )
        .into_boxed(),
        FacSurfaceCreateEntity::new_rail_straight_facto(
            start.move_xy(6, 4).to_f32(),
            FactoDirection::NorthEast,
        )
        .into_boxed(),
        FacSurfaceCreateEntity::new_rail_curved_facto(
            start.move_xy(3, 1).to_f32(),
            FactoDirection::SouthEast,
        )
        .into_boxed(),
    ]
}

pub fn rail_degrees_south(start: VPoint) -> Vec<Box<dyn LuaCommand>> {
    start.assert_odd_position();
    vec![
        FacSurfaceCreateEntity::new_rail_curved_facto(
            start.move_xy(1, 3).to_f32(),
            FactoDirection::South,
        )
        .into_boxed(),
        FacSurfaceCreateEntity::new_rail_straight_facto(
            start.move_xy(4, 6).to_f32(),
            FactoDirection::SouthWest,
        )
        .into_boxed(),
        FacSurfaceCreateEntity::new_rail_curved_facto(
            start.move_xy(7, 9).to_f32(),
            FactoDirection::NorthWest,
        )
        .into_boxed(),
    ]
}

pub fn rail_degrees_west(start: VPoint) -> Vec<Box<dyn LuaCommand>> {
    start.assert_odd_position();
    vec![
        FacSurfaceCreateEntity::new_rail_curved_facto(
            start.move_xy(7, 1).to_f32(),
            FactoDirection::West,
        )
        .into_boxed(),
        FacSurfaceCreateEntity::new_rail_straight_facto(
            start.move_xy(4, 4).to_f32(),
            FactoDirection::NorthWest,
        )
        .into_boxed(),
        FacSurfaceCreateEntity::new_rail_curved_facto(
            start.move_xy(1, 7).to_f32(),
            FactoDirection::NorthEast, // wtf?
        )
        .into_boxed(),
    ]
}

pub fn rail_degrees_east(start: VPoint) -> Vec<Box<dyn LuaCommand>> {
    start.assert_odd_position();
    vec![
        FacSurfaceCreateEntity::new_rail_curved_facto(
            start.move_xy(3, 9).to_f32(),
            FactoDirection::East,
        )
        .into_boxed(),
        FacSurfaceCreateEntity::new_rail_straight_facto(
            start.move_xy(6, 6).to_f32(),
            FactoDirection::SouthEast,
        )
        .into_boxed(),
        FacSurfaceCreateEntity::new_rail_curved_facto(
            start.move_xy(9, 3).to_f32(),
            FactoDirection::SouthWest,
        )
        .into_boxed(),
    ]
}

pub fn dual_rail_north(top_right_corner: VPoint, commands: &mut Vec<Box<dyn LuaCommand>>) {
    top_right_corner.assert_odd_position();

    commands.append(&mut rail_degrees_north(top_right_corner.move_xy(4, 0)));
    commands.append(&mut rail_degrees_north(top_right_corner.move_xy(0, 4)));

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

pub const fn dual_rail_north_empty() -> StaticBitGrid {
    const INNER: [bool; GRID_16X16_SIZE] = [
        false, false, false, false, false, false, false, false, true, true, true, true, true, true,
        true, true, false, false, false, false, false, false, false, false, false, false, true,
        true, true, true, true, true, true, true, true, true, true, true, true, false, false,
        false, false, true, true, true, true, true, true, true, true, true, true, true, true, true,
        false, false, false, false, true, true, true, true, false, false, false, false, true, true,
        true, true, true, true, false, false, false, true, true, true, false, false, false, false,
        false, false, true, true, true, true, true, false, false, false, true, true, true, true,
        true, false, false, false, false, true, true, true, true, true, false, false, false, true,
        true, true, true, true, false, false, false, false, true, true, true, true, false, false,
        false, true, true, true, true, true, true, true, false, false, false, true, true, true,
        true, false, false, false, true, true, true, true, true, true, true, false, false, false,
        true, true, true, true, false, false, true, true, true, true, true, true, true, true,
        false, false, false, true, true, true, false, false, true, true, true, true, true, true,
        true, true, false, false, false, true, true, true, false, false, true, true, true, true,
        true, true, true, true, true, false, false, false, true, true, false, false, true, true,
        true, true, true, true, true, true, true, true, false, false, true, true, false, false,
        true, true, true, true, true, true, true, true, true, true, false, false, true, true,
        false, false, true, true, true, true, true, true, true, true, true, true, false, false,
        true, true, false, false,
    ];
    const GRID: StaticBitGrid = StaticBitGrid::new(INNER);
    GRID
}

pub fn dual_rail_south(top_right_corner: VPoint, commands: &mut Vec<Box<dyn LuaCommand>>) {
    top_right_corner.assert_odd_position();

    commands.append(&mut rail_degrees_south(top_right_corner.move_xy(4, 0)));
    commands.append(&mut rail_degrees_south(top_right_corner.move_xy(0, 4)));

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

pub const fn dual_rail_south_empty() -> StaticBitGrid {
    const INNER: [bool; GRID_16X16_SIZE] = [
        false, false, true, true, false, false, true, true, true, true, true, true, true, true,
        true, true, false, false, true, true, false, false, true, true, true, true, true, true,
        true, true, true, true, false, false, true, true, false, false, true, true, true, true,
        true, true, true, true, true, true, false, false, true, true, false, false, false, true,
        true, true, true, true, true, true, true, true, false, false, true, true, true, false,
        false, false, true, true, true, true, true, true, true, true, false, false, true, true,
        true, false, false, false, true, true, true, true, true, true, true, true, false, false,
        true, true, true, true, false, false, false, true, true, true, true, true, true, true,
        false, false, false, true, true, true, true, false, false, false, true, true, true, true,
        true, true, true, false, false, false, true, true, true, true, false, false, false, false,
        true, true, true, true, true, false, false, false, true, true, true, true, true, false,
        false, false, false, true, true, true, true, true, false, false, false, true, true, true,
        true, true, false, false, false, false, false, false, true, true, true, false, false,
        false, true, true, true, true, true, true, false, false, false, false, true, true, true,
        true, false, false, false, false, true, true, true, true, true, true, true, true, true,
        true, true, true, true, false, false, false, false, true, true, true, true, true, true,
        true, true, true, true, true, true, true, false, false, false, false, false, false, false,
        false, false, false, true, true, true, true, true, true, true, true, false, false, false,
        false, false, false, false, false,
    ];
    const GRID: StaticBitGrid = StaticBitGrid::new(INNER);
    GRID
}

pub fn dual_rail_east(top_right_corner: VPoint, commands: &mut Vec<Box<dyn LuaCommand>>) {
    top_right_corner.assert_odd_position();

    commands.append(&mut rail_degrees_east(top_right_corner.move_xy(4, 4)));
    commands.append(&mut rail_degrees_east(top_right_corner.move_xy(0, 0)));

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

pub const fn dual_rail_east_empty() -> StaticBitGrid {
    const INNER: [bool; GRID_16X16_SIZE] = [
        true, true, true, true, true, true, true, true, true, true, false, false, true, true,
        false, false, true, true, true, true, true, true, true, true, true, true, false, false,
        true, true, false, false, true, true, true, true, true, true, true, true, true, true,
        false, false, true, true, false, false, true, true, true, true, true, true, true, true,
        true, false, false, false, true, true, false, false, true, true, true, true, true, true,
        true, true, false, false, false, true, true, true, false, false, true, true, true, true,
        true, true, true, true, false, false, false, true, true, true, false, false, true, true,
        true, true, true, true, true, false, false, false, true, true, true, true, false, false,
        true, true, true, true, true, true, false, false, false, true, true, true, true, false,
        false, false, true, true, true, true, false, false, false, false, true, true, true, true,
        false, false, false, true, true, true, true, false, false, false, false, true, true, true,
        true, true, false, false, false, true, false, false, false, false, false, false, true,
        true, true, true, true, false, false, false, true, true, false, false, false, false, true,
        true, true, true, true, true, false, false, false, true, true, true, true, true, true,
        true, true, true, true, true, false, false, false, false, true, true, true, true, true,
        true, true, true, true, true, true, false, false, false, false, true, true, true, true,
        true, false, false, false, false, false, false, false, false, false, false, true, true,
        true, true, true, true, false, false, false, false, false, false, false, false, true, true,
        true, true, true, true, true, true,
    ];
    const GRID: StaticBitGrid = StaticBitGrid::new(INNER);
    GRID
}

pub fn dual_rail_west(top_right_corner: VPoint, commands: &mut Vec<Box<dyn LuaCommand>>) {
    top_right_corner.assert_odd_position();

    commands.append(&mut rail_degrees_west(top_right_corner.move_xy(4, 4)));
    commands.append(&mut rail_degrees_west(top_right_corner.move_xy(0, 0)));

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

pub const fn dual_rail_west_empty() -> StaticBitGrid {
    const INNER: [bool; GRID_16X16_SIZE] = [
        true, true, true, true, true, true, true, true, false, false, false, false, false, false,
        false, false, true, true, true, true, true, true, false, false, false, false, false, false,
        false, false, false, false, true, true, true, true, true, false, false, false, false, true,
        true, true, true, true, true, true, true, true, true, true, false, false, false, false,
        true, true, true, true, true, true, true, true, true, true, true, false, false, false,
        true, true, true, true, true, true, false, false, false, false, true, true, false, false,
        false, true, true, true, true, true, false, false, false, false, false, false, true, false,
        false, false, true, true, true, true, true, false, false, false, false, true, true, true,
        true, false, false, false, true, true, true, true, false, false, false, false, true, true,
        true, true, false, false, false, true, true, true, true, false, false, false, true, true,
        true, true, true, true, false, false, true, true, true, true, false, false, false, true,
        true, true, true, true, true, true, false, false, true, true, true, false, false, false,
        true, true, true, true, true, true, true, true, false, false, true, true, true, false,
        false, false, true, true, true, true, true, true, true, true, false, false, true, true,
        false, false, false, true, true, true, true, true, true, true, true, true, false, false,
        true, true, false, false, true, true, true, true, true, true, true, true, true, true,
        false, false, true, true, false, false, true, true, true, true, true, true, true, true,
        true, true, false, false, true, true, false, false, true, true, true, true, true, true,
        true, true, true, true,
    ];
    const GRID: StaticBitGrid = StaticBitGrid::new(INNER);
    GRID
}

fn add_rail(point: VPoint, direction: RailDirection, commands: &mut Vec<Box<dyn LuaCommand>>) {
    commands
        .push(FacSurfaceCreateEntity::new_rail_straight(point.to_f32(), direction).into_boxed());
}

pub fn dual_rail_empty_index_to_xy(grid: &[bool; GRID_16X16_SIZE], index: usize) -> VPoint {
    assert!(index < GRID_16X16_SIZE, "too big {}", index,);

    let diameter_component = index - (index % 16);
    let y = diameter_component / 16;
    let x = index - diameter_component;
    VPoint::new(x as i32, y as i32)
}
