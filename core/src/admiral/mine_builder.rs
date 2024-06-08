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
    let mine_areas = surface.get_mine_areas().collect_vec();

    let radius = surface.get_radius_i32();
    let range = -radius..radius;
    let mut commands = Vec::new();
    const X_CHUNK: i32 = 4;
    const Y_CHUNK: i32 = 3;
    for x in range.clone() {
        for y in range.clone() {
            if x % X_CHUNK != 0 || y % Y_CHUNK != 0 {
                continue;
            }
            let point = VPoint::new(x, y);
            if !mine_areas.iter().any(|v| v.contains_point(&point)) {
                continue;
            }

            let mut found = false;
            'outer: for drill_x in x..(x + 3) {
                for drill_y in y..(y + 3) {
                    let drill_point = VPoint::new(drill_x, drill_y);
                    if surface.get_pixel(drill_point).is_resource() {
                        found = true;
                        break 'outer;
                    }
                }
            }
            if !found {
                continue;
            }

            commands.push(
                FacSurfaceCreateEntity::new_drill(
                    point.to_f32_with_offset(1.5),
                    RailDirection::Right,
                )
                .into_boxed(),
            );
            commands.push(
                FacSurfaceCreateEntity::new_chest_red(point.move_xy(3, 1).to_f32_with_offset(0.5))
                    .into_boxed(),
            );

            if x % (X_CHUNK * 2) == 0 && y % (Y_CHUNK * 2) == 0 {
                commands.push(
                    FacSurfaceCreateEntity::new_electric_pole_medium(
                        point.move_x(3).to_f32_with_offset(0.5),
                    )
                    .into_boxed(),
                );
            }
        }
    }

    admiral
        .execute_checked_commands_in_wrapper_function(commands)
        .unwrap();

    chart_pulse(admiral, surface.get_radius())?;

    Ok(())
}
