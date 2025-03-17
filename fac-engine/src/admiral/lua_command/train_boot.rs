use super::{DEFAULT_SURFACE_VAR, LuaCommand, raw_lua::RawLuaCommand};
use crate::admiral::err::AdmiralResult;
use crate::blueprint::output::FacItemOutput;
use crate::common::varea::{VArea, VAreaSugar};
use crate::util::ansi::C_BLOCK_LINE;
use std::rc::Rc;

pub fn train_boot(area: VArea, output: Rc<FacItemOutput>) -> AdmiralResult<()> {
    let VAreaSugar {
        start_x,
        start_y,
        end_x,
        end_y,
    } = area.desugar();
    let command = RawLuaCommand::new(format!(
        r#"
        local found_pos = ""
        local is_found = false
        for _,train in pairs({DEFAULT_SURFACE_VAR}.get_trains()) do
            engine = train.carriages[1]
            found_pos = found_pos .. "found " .. engine.position.x .. "{C_BLOCK_LINE}" .. engine.position.x .. " "   
            if engine.position.x >= {start_x} and engine.position.x <= {end_x} and engine.position.y >= {start_y} and engine.position.x <= {end_y} then
                is_found = true
                train.manual_mode = false
            end
        end
        if not is_found then
            rcon.print("No train found between {start_x}{C_BLOCK_LINE}{start_y} and {end_x}{C_BLOCK_LINE}{end_y}")
            rcon.print(found_pos)
        end
        "#
    ));

    // must synthesize the train first
    output.flush();
    let _ = output.admiral_execute_command(command.into_boxed())?;

    Ok(())
}
