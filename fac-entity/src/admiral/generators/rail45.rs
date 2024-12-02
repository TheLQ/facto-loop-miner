use crate::admiral::lua_command::LuaCommand;
use crate::admiral::lua_command::fac_surface_create_entity::FacSurfaceCreateEntity;
use crate::common::vpoint::VPoint;
use crate::navigator::mori::{Rail, RailDirection, TurnType};

pub fn rail_45(
    result: &mut Vec<Box<dyn LuaCommand>>,
    start: Rail,
    directions: [FacDirectionQuarter; 2],
    turn_type: TurnType,
    sections: usize,
) -> Rail {
    let mut cur_point = start;
    for _ in 0..sections {
        result.push(
            FacSurfaceCreateEntity::new_rail_straight_facto(
                cur_point.endpoint.to_f32(),
                directions[0].clone(),
            )
            .into_boxed(),
        );
        cur_point = cur_point.move_forward_micro_num(2);
        result.push(
            FacSurfaceCreateEntity::new_rail_straight_facto(
                cur_point.endpoint.to_f32(),
                directions[1].clone(),
            )
            .into_boxed(),
        );
        cur_point = cur_point
            .move_force_rotate_clockwise(turn_type.rotations())
            .move_forward_micro_num(2)
            .move_force_rotate_clockwise(turn_type.swap().rotations());
    }
    cur_point
}

/// Start FROM top left!
pub fn rail_45_down(
    result: &mut Vec<Box<dyn LuaCommand>>,
    start_point: VPoint,
    angle_sections: usize,
) {
    let start = Rail::new_straight(start_point, RailDirection::Left);

    // third 45 turn back down
    result.push(
        FacSurfaceCreateEntity::new_rail_curved_facto(
            start
                .move_forward_micro_num(5)
                .move_force_rotate_clockwise(1)
                .move_forward_micro_num(1)
                .endpoint
                .to_f32(),
            FacDirectionQuarter::West,
        )
        .into_boxed(),
    );

    // straight 45 down
    let end_of_45 = rail_45(
        result,
        start
            .move_forward_micro_num(8)
            .move_force_rotate_clockwise(1)
            .move_forward_micro_num(4)
            .move_force_rotate_clockwise(3),
        [
            FacDirectionQuarter::NorthWest,
            FacDirectionQuarter::SouthEast,
        ],
        TurnType::Turn90,
        angle_sections,
    );

    // ending 45 curve to normal straight
    let straight_lead = end_of_45
        .move_forward_micro_num(3)
        .move_force_rotate_clockwise(1)
        .move_forward_micro_num(1)
        .move_force_rotate_clockwise(3);
    result.push(
        FacSurfaceCreateEntity::new_rail_curved_facto(
            straight_lead.endpoint.to_f32(),
            FacDirectionQuarter::East,
        )
        .into_boxed(),
    );
}

/// Start FROM top left!
pub fn rail_45_up(
    result: &mut Vec<Box<dyn LuaCommand>>,
    start_point: VPoint,
    angle_sections: usize,
) -> Rail {
    let start = Rail::new_straight(start_point, RailDirection::Left);

    // third 45 turn back down
    result.push(
        FacSurfaceCreateEntity::new_rail_curved_facto(
            start
                .move_forward_micro_num(5)
                .move_force_rotate_clockwise(1)
                .move_forward_micro_num(1)
                .endpoint
                .to_f32(),
            FacDirectionQuarter::NorthWest,
        )
        .into_boxed(),
    );

    // straight 45 down
    let end_of_45 = rail_45(
        result,
        start
            .move_forward_micro_num(8)
            .move_force_rotate_clockwise(3)
            .move_forward_micro_num(2)
            .move_force_rotate_clockwise(1),
        [
            FacDirectionQuarter::SouthWest,
            FacDirectionQuarter::NorthEast,
        ],
        TurnType::Turn270,
        angle_sections,
    );

    // ending 45 curve to normal straight
    let straight_lead = end_of_45
        .move_forward_micro_num(3)
        .move_force_rotate_clockwise(3)
        .move_forward_micro_num(1)
        .move_force_rotate_clockwise(1);
    result.push(
        FacSurfaceCreateEntity::new_rail_curved_facto(
            straight_lead.endpoint.to_f32(),
            FacDirectionQuarter::SouthEast,
        )
        .into_boxed(),
    );

    end_of_45
}
