use crate::admiral::err::{pretty_panic_admiral, AdmiralError, AdmiralResult};
use crate::admiral::executor::client::AdmiralClient;
use crate::admiral::executor::LuaCompiler;
use crate::admiral::generators::rail90::{
    rail_degrees_east, rail_degrees_north, rail_degrees_south, rail_degrees_west,
};
use crate::admiral::lua_command::fac_destroy::FacDestroy;
use crate::admiral::lua_command::fac_log::FacLog;
use crate::admiral::lua_command::lua_batch::LuaBatchCommand;
use crate::admiral::lua_command::raw_lua::RawLuaCommand;
use crate::admiral::lua_command::scanner::{
    facscan_hyper_scan, facscan_mega_export_entities_compressed, BaseScanner,
};
use crate::admiral::lua_command::LuaCommand;
use crate::state::machine_v1::CROP_RADIUS;
use crate::surfacev::vpoint::VPoint;
use tracing::info;

pub fn admiral_entrypoint(mut admiral: AdmiralClient) {
    info!("admiral entrypoint");

    match 1 {
        1 => admiral_entrypoint_testing(&mut admiral)
            .map_err(pretty_panic_admiral)
            .unwrap(),
        _ => panic!("asdf"),
    }

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
    scan_area(admiral)?;
    destroy_placed_entities(admiral)?;

    Ok(())
}

fn scan_area(admiral: &mut AdmiralClient) -> AdmiralResult<()> {
    // Need to have generated space for our testing
    let command = BaseScanner::new_radius(CROP_RADIUS);
    admiral.execute_checked_command(command.into_boxed())?;

    Ok(())
}

fn destroy_placed_entities(admiral: &mut AdmiralClient) -> AdmiralResult<()> {
    let command = FacDestroy::new_filtered(150, vec!["straight-rail", "curved-rail"]);
    admiral.execute_checked_command(command.into_boxed())?;

    Ok(())
}

fn admiral_entrypoint_testing(admiral: &mut AdmiralClient) -> AdmiralResult<()> {
    scan_area(admiral)?;
    destroy_placed_entities(admiral)?;

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
