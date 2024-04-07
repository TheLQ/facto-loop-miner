use crate::admiral::lua_command::LuaCommand;

#[derive(Debug)]
pub struct FacDestroy {
    radius: u32,
}

impl FacDestroy {
    pub fn new(radius: u32) -> Self {
        Self { radius }
    }
}

impl LuaCommand for FacDestroy {
    fn make_lua(&self) -> String {
        // game.players[1].teleport({{ 1000, 1000 }})
        // rcon.print('destroy_' .. entity.name )
        // for entity in
        let radius = self.radius;
        format!(r"
local entities = game.surfaces[1].find_entities({{ {{ -{radius}, -{radius} }} , {{ {radius}, {radius} }} }})
for _, entity in ipairs(entities) do
    entity.destroy()
end
        ")
        .to_string()
    }
}
