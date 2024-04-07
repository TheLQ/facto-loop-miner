use crate::admiral::executor::client::AdmiralClient;
use crate::admiral::executor::LuaCompiler;
use crate::admiral::lua_command::fac_log::FacLog;
use crate::admiral::lua_command::raw_lua::RawLuaCommand;
use crate::admiral::lua_command::scanner::{
    facscan_hyper_scan, facscan_mega_export_entities_compressed,
};
use tracing::info;

pub fn admiral_entrypoint(mut admiral: AdmiralClient) {
    info!("admiral entrypoint");

    // let res = admiral
    //     ._execute_statement(RawLuaCommand::new(
    //         "game.surfaces[1].create_entity()".to_string(),
    //     ))
    //     .unwrap();
    let res = admiral
        ._execute_statement(RawLuaCommand::new(
            "game.surfaces[1].create_entity{ name = \"steel-chest\", position = { 999999,999999 } } rcon.print('asdf')".to_string(),
        ))
        .unwrap();
    info!("response {}", res.body);

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
