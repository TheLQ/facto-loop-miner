use crate::admiral::executor::client::AdmiralClient;
use crate::admiral::executor::LuaCompiler;
use crate::admiral::lua_command::fac_log::FacLog;
use crate::admiral::lua_command::raw_lua::RawLuaCommand;
use crate::admiral::lua_command::scanner::{
    facscan_hyper_scan, facscan_mega_export_entities_compressed, BaseScanner,
};
use crate::admiral::lua_command::LuaCommand;
use crate::state::machine_v1::CROP_RADIUS;
use tracing::info;

pub fn admiral_entrypoint(mut admiral: AdmiralClient) {
    info!("admiral entrypoint");

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

fn admiral_entrypoint_testing(mut admiral: AdmiralClient) {
    // Need to have generated space for our testing
    admiral
        .execute_checked_command(BaseScanner::new_radius(CROP_RADIUS).into_boxed())
        .unwrap();
    
    90
}