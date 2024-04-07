use crate::admiral::lua_command::LuaCommand;
use itertools::Itertools;

#[derive(Debug)]
pub struct FacDestroy {
    radius: u32,
    entity_names: Vec<&'static str>,
}

impl FacDestroy {
    pub fn new(radius: u32) -> Self {
        Self {
            radius,
            entity_names: Vec::new(),
        }
    }

    pub fn new_filtered(radius: u32, entity_names: Vec<&'static str>) -> Self {
        Self {
            radius,
            entity_names,
        }
    }
}

impl LuaCommand for FacDestroy {
    fn make_lua(&self) -> String {
        // game.players[1].teleport({{ 1000, 1000 }})
        // rcon.print('destroy_' .. entity.name )
        // for entity in
        let radius = self.radius;
        let filters = self
            .entity_names
            .iter()
            .map(|v| format!("\"{}\"", v))
            .join(",");
        format!(r"
local entities = game.surfaces[1].find_entities_filtered{{ {{ {{ -{radius}, -{radius} }} , {{ {radius}, {radius} }} }}, {{ {filters} }} }}
for _, entity in ipairs(entities) do
    entity.destroy()
end
        ")
            .trim().replace('\n', "")
    }
}
