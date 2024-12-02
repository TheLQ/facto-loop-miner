use crate::admiral::err::AdmiralResult;
use crate::admiral::executor::client::AdmiralClient;
use crate::admiral::executor::entrypoint::chart_pulse;
use crate::admiral::executor::LuaCompiler;
use crate::admiral::lua_command::fac_destroy::FacDestroy;
use crate::admiral::lua_command::fac_surface_create_entity::FacSurfaceCreateEntity;
use crate::admiral::lua_command::LuaCommand;
use crate::navigator::mori::RailDirection;
use crate::surfacev::vpoint::VPoint;
use crate::surfacev::vsurface::VSurface;
use itertools::Itertools;
use std::path::Path;

pub fn admiral_mines(admiral: &mut AdmiralClient) -> AdmiralResult<()> {
    let step = "step21-demark";
    let surface = VSurface::load(&Path::new("work/out0").join(step))?;
    let radius = surface.get_radius();

    admiral.execute_checked_command(
        FacDestroy::new_filtered(
            radius,
            vec![
                "logistic-chest-passive-provider",
                "medium-electric-pole",
                "electric-mining-drill",
                "small-lamp",
            ],
        )
        .into_boxed(),
    )?;

    insert_mines(admiral, &surface)
}

fn insert_mines(admiral: &mut AdmiralClient, surface: &VSurface) -> AdmiralResult<()> {
    const X_CHUNK: i32 = 4;
    const Y_CHUNK: i32 = 3;

    let mut commands = Vec::new();
    let mines = surface.get_mines();
    for mine in mines {
        for patch in mine.mine_base.get_vpatches(surface) {
            // let mut patch_start = patch.area.start;
            // patch_start = patch_start.move_xy(patch_start.x() % X_CHUNK, patch_start.y() % Y_CHUNK);
            for point in patch.area.get_points() {
                if point.x() % 7 != 0 || point.y() % Y_CHUNK != 0 {
                    continue;
                }

                let drill_left_point = point;
                let drill_right_point = point.move_xy(4, 0);

                let drill_left_useful = is_drill_useful_at_point(surface, &drill_left_point);
                let drill_right_useful = is_drill_useful_at_point(surface, &drill_right_point);
                if !drill_left_useful && !drill_right_useful {
                    continue;
                }

                if drill_left_useful {
                    commands.push(
                        FacSurfaceCreateEntity::new_drill(
                            drill_left_point.to_f32_with_offset(1.5),
                            RailDirection::Right,
                        )
                        .into_boxed(),
                    );
                }
                commands.push(
                    FacSurfaceCreateEntity::new_chest_red(
                        point.move_xy(3, 1).to_f32_with_offset(0.5),
                    )
                    .into_boxed(),
                );
                if drill_right_useful {
                    commands.push(
                        FacSurfaceCreateEntity::new_drill(
                            drill_right_point.to_f32_with_offset(1.5),
                            RailDirection::Left,
                        )
                        .into_boxed(),
                    );
                }

                // if point.x() % (X_CHUNK * 2) == 0 && point.y() % (Y_CHUNK * 2) == 0 {
                //     commands.push(
                //         FacSurfaceCreateEntity::new_electric_pole_medium(
                //             point.move_x(3).to_f32_with_offset(0.5),
                //         )
                //         .into_boxed(),
                //     );
                // }
            }
        }
    }

    admiral
        .execute_checked_commands_in_wrapper_function(commands)
        .unwrap();

    chart_pulse(admiral, surface.get_radius())?;

    Ok(())
}

fn is_drill_useful_at_point(surface: &VSurface, start: &VPoint) -> bool {
    for drill_x in start.x()..(start.x() + 3) {
        for drill_y in start.y()..(start.y() + 3) {
            let drill_point = VPoint::new(drill_x, drill_y);
            if surface.get_pixel(drill_point).is_resource() {
                return true;
            }
        }
    }
    false
}
