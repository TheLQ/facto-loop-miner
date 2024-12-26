use crate::common::varea::VArea;

use super::{DEFAULT_SURFACE_VAR, raw_lua::RawLuaCommand};

pub fn train_boot(area: VArea) -> RawLuaCommand {
    let (x_min, x_max, y_min, y_max) = area.desugar();
    RawLuaCommand::new(format!(
        r#"
        for _,train in pairs({DEFAULT_SURFACE_VAR}.get_trains()) do
            engine = train.carriages[1]
            if engine.position.x >= {x_min} and engine.position.x <= {x_max} and engine.position.y >= {y_min} and engine.position.x <= {y_max} then
                train.manual_mode = false
            end
        end
        "#
    ))
}
