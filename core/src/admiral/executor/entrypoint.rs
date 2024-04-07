use crate::admiral::err::AdmiralResult;
use crate::admiral::executor::client::AdmiralClient;
use crate::admiral::executor::LuaCompiler;
use crate::admiral::generators::rail90::{
    rail_degrees_180, rail_degrees_270, rail_degrees_360, rail_degrees_90,
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
        1 => admiral_entrypoint_testing(&mut admiral).unwrap(),
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

fn admiral_entrypoint_testing(admiral: &mut AdmiralClient) -> AdmiralResult<()> {
    {
        // Need to have generated space for our testing
        let command = BaseScanner::new_radius(CROP_RADIUS);
        admiral.execute_checked_command(command.into_boxed())?;
    }

    {
        let command = FacDestroy::new(150);
        admiral.execute_checked_command(command.into_boxed())?;
    }

    {
        let command = rail_degrees_90(VPoint::new(0, 0).to_f32_with_offset(0.0));
        let command = LuaBatchCommand::new(Vec::from(command));
        admiral.execute_checked_command(command.into_boxed())?;

        let command = rail_degrees_180(VPoint::new(32, 0).to_f32_with_offset(0.0));
        let command = LuaBatchCommand::new(Vec::from(command));
        admiral.execute_checked_command(command.into_boxed())?;

        let command = rail_degrees_270(VPoint::new(64, 0).to_f32_with_offset(0.0));
        let command = LuaBatchCommand::new(Vec::from(command));
        admiral.execute_checked_command(command.into_boxed())?;

        let command = rail_degrees_360(VPoint::new(96, 0).to_f32_with_offset(0.0));
        let command = LuaBatchCommand::new(Vec::from(command));
        admiral.execute_checked_command(command.into_boxed())?;
    }

    Ok(())
}
