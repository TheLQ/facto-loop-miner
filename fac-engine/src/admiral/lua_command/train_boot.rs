use super::{DEFAULT_SURFACE_VAR, LuaCommand, raw_lua::RawLuaCommand};
use crate::admiral::err::AdmiralResult;
use crate::blueprint::output::FacItemOutput;
use crate::common::varea::VArea;
use crate::util::ansi::C_BLOCK_LINE;
use std::rc::Rc;

pub fn train_boot(area: VArea, output: Rc<FacItemOutput>) -> AdmiralResult<()> {
    let (x_min, x_max, y_min, y_max) = area.desugar();
    let command = RawLuaCommand::new(format!(
        r#"
        local found_pos = ""
        local is_found = false
        for _,train in pairs({DEFAULT_SURFACE_VAR}.get_trains()) do
            engine = train.carriages[1]
            found_pos = found_pos .. "found " .. engine.position.x .. "{C_BLOCK_LINE}" .. engine.position.x .. " "   
            if engine.position.x >= {x_min} and engine.position.x <= {x_max} and engine.position.y >= {y_min} and engine.position.x <= {y_max} then
                is_found = true
                train.manual_mode = false
            end
        end
        if not is_found then
            rcon.print("No train found between {x_min}{C_BLOCK_LINE}{y_min} and {x_max}{C_BLOCK_LINE}{y_max}")
            rcon.print(found_pos)
        end
        "#
    ));

    // must synthesize the train first
    output.flush();
    let _ = output.admiral_execute_command(command.into_boxed())?;

    Ok(())
}
