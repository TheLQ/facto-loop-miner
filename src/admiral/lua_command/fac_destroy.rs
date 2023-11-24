use crate::admiral::lua_command::LuaCommand;

#[derive(Debug)]
pub struct FacDestroy {}

impl LuaCommand for FacDestroy {
    fn make_lua(&self) -> String {
        // for entity in
        format!(
            r#"
local entities = game.surfaces[1].find_entities({{ {{ 0,0 }} , {{ 10000, 10000 }} }})
for _, entity in ipairs(entities) do
    rcon.print('destroy_' .. entity.name )
    entity.destroy()
end
rcon.print('destroy_success')
        "# // game.players[1].teleport({{ 1000, 1000 }})
        )
    }
}
