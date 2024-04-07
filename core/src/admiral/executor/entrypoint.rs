use crate::admiral::err::{pretty_panic_admiral, AdmiralError, AdmiralResult};
use crate::admiral::executor::client::AdmiralClient;
use crate::admiral::executor::LuaCompiler;
use crate::admiral::generators::rail90::{
    rail_degrees_east, rail_degrees_north, rail_degrees_south, rail_degrees_west,
};
use crate::admiral::lua_command::chart_pulse::ChartPulse;
use crate::admiral::lua_command::fac_destroy::FacDestroy;
use crate::admiral::lua_command::fac_log::FacLog;
use crate::admiral::lua_command::fac_surface_create_entity::FacSurfaceCreateEntity;
use crate::admiral::lua_command::lua_batch::LuaBatchCommand;
use crate::admiral::lua_command::raw_lua::RawLuaCommand;
use crate::admiral::lua_command::scanner::{
    facscan_hyper_scan, facscan_mega_export_entities_compressed, BaseScanner,
};
use crate::admiral::lua_command::LuaCommand;
use crate::navigator::mori::RailMode;
use crate::state::machine_v1::{CROP_RADIUS, REMOVE_RESOURCE_BASE_TILES};
use crate::surface::surface::Surface;
use crate::surfacev::vpoint::VPoint;
use crate::surfacev::vsurface::VSurface;
use std::path::Path;
use tracing::info;

pub fn admiral_entrypoint(mut admiral: AdmiralClient) {
    info!("admiral entrypoint");

    match 2 {
        1 => admiral_entrypoint_testing(&mut admiral),
        2 => admiral_entrypoint_prod(&mut admiral),

        _ => panic!("asdf"),
    }
    .map_err(pretty_panic_admiral)
    .unwrap();

    // validate we have space for us

    // for command in facscan_hyper_scan() {
    //     let res = admiral._execute_statement(command).unwrap();
    //     info!("return: {}", res.body);
    // }
    // for command in facscan_mega_export_entities_compressed() {
    //     let res = admiral._execute_statement(command).unwrap();
    //     info!("return: {}", res.body);
    // }
    // let res = admiral
    //     ._execute_statement(FacLog::new("done".to_string()))
    //     .unwrap();
    // info!("return: {}", res.body);
}

fn admiral_entrypoint_prod(admiral: &mut AdmiralClient) -> AdmiralResult<()> {
    // if 1 + 1 == 2 {
    //     return Ok(());
    // }

    let surface = VSurface::load(Path::new("work/out0/step20-nav"))?;
    let radius = surface.get_radius();

    scan_area(admiral, radius)?;
    destroy_placed_entities(admiral, radius)?;

    insert_rail_from_surface(admiral, &surface)?;

    chart_pulse(admiral, radius)?;

    Ok(())
}

fn scan_area(admiral: &mut AdmiralClient, radius: u32) -> AdmiralResult<()> {
    // Need to have generated space for our testing
    let command = BaseScanner::new_radius(radius);
    admiral.execute_checked_command(command.into_boxed())?;

    Ok(())
}

fn destroy_placed_entities(admiral: &mut AdmiralClient, radius: u32) -> AdmiralResult<()> {
    let command = FacDestroy::new_filtered(radius, vec!["straight-rail", "curved-rail"]);
    admiral.execute_checked_command(command.into_boxed())?;

    Ok(())
}

fn insert_rail_from_surface(admiral: &mut AdmiralClient, surface: &VSurface) -> AdmiralResult<()> {
    let mut entities = Vec::new();

    for rail in surface.get_rail() {
        // if !rail
        //     .endpoint
        //     .is_within_center_radius(REMOVE_RESOURCE_BASE_TILES as u32 + 20)
        // {
        //     continue;
        // }
        if rail.mode != RailMode::Straight {
            continue;
        }
        info!("writing {:?}", rail);

        rail.to_factorio_entities(&mut entities);
    }

    let entities_length = entities.len();
    admiral.execute_checked_commands_in_wrapper_function(entities)?;
    info!("Inserted {} rail", entities_length);

    // let command = FacSurfaceCreateEntity::new_rail_straight(
    //     rail.endpoint.to_f32_with_offset(1.0),
    //     rail.direction.clone(),
    // );
    // admiral.execute_checked_command(command.into_boxed())?;
    Ok(())
}

fn chart_pulse(admiral: &mut AdmiralClient, radius: u32) -> AdmiralResult<()> {
    let command = ChartPulse::new_radius(radius);
    admiral.execute_checked_command(command.into_boxed())?;

    Ok(())
}

fn admiral_entrypoint_testing(admiral: &mut AdmiralClient) -> AdmiralResult<()> {
    const WORK_RADIUS: u32 = REMOVE_RESOURCE_BASE_TILES as u32 + 20;

    scan_area(admiral, WORK_RADIUS)?;
    destroy_placed_entities(admiral, WORK_RADIUS)?;

    {
        let command = rail_degrees_south(VPoint::new(0, 0).to_f32_with_offset(0.0));
        let command = LuaBatchCommand::new(Vec::from(command));
        admiral.execute_checked_command(command.into_boxed())?;

        let command = rail_degrees_west(VPoint::new(32, 0).to_f32_with_offset(0.0));
        let command = LuaBatchCommand::new(Vec::from(command));
        admiral.execute_checked_command(command.into_boxed())?;

        let command = rail_degrees_north(VPoint::new(64, 0).to_f32_with_offset(0.0));
        let command = LuaBatchCommand::new(Vec::from(command));
        admiral.execute_checked_command(command.into_boxed())?;

        let command = rail_degrees_east(VPoint::new(96, 0).to_f32_with_offset(0.0));
        let command = LuaBatchCommand::new(Vec::from(command));
        admiral.execute_checked_command(command.into_boxed())?;
    }

    Ok(())
}

pub fn testing_rail_turns() {}
