use crate::admiral::lua_command::LuaCommand;
use crate::common::varea::VArea;
use itertools::Itertools;
use std::borrow::Borrow;
use tracing::debug;

#[derive(Debug)]
pub struct FacDestroy {
    area: String,
    entity_names: Vec<String>,
}

impl FacDestroy {
    pub fn new_everything(radius: u32) -> Self {
        Self {
            area: format!("{{ {{ -{radius}, -{radius} }} , {{ {radius}, {radius} }} }}"),
            entity_names: Vec::new(),
        }
    }

    pub fn new_filtered(radius: u32, entity_names: Vec<String>) -> Self {
        if entity_names.is_empty() {
            panic!("empty entities, not destroying everything")
        }
        Self {
            area: format!("{{ {{ -{radius}, -{radius} }} , {{ {radius}, {radius} }} }}"),
            entity_names,
        }
    }

    pub fn new_filtered_area(area: impl Borrow<VArea>, entity_names: Vec<String>) -> Self {
        if entity_names.is_empty() {
            panic!("empty entities, not destroying everything")
        }
        let area = area.borrow();
        let start_x = area.start.x();
        let start_y = area.start.y();
        let end_x = area.point_bottom_left().x();
        let end_y = area.point_bottom_left().y();
        Self {
            area: format!("{{ {{ {start_x}, {start_y} }} , {{ {end_x}, {end_y} }} }}"),
            entity_names,
        }
    }
}

impl LuaCommand for FacDestroy {
    fn make_lua(&self) -> String {
        if self.entity_names.is_empty() {
            self.destroy_everything()
        } else {
            self.destroy_filtered()
        }
    }
}

impl FacDestroy {
    fn destroy_filtered(&self) -> String {
        debug!("destroying filtered {:?}", self);
        // game.players[1].teleport({{ 1000, 1000 }})
        // rcon.print('destroy_' .. entity.name )
        // for entity in
        let area = &self.area;
        let filters = self
            .entity_names
            .iter()
            .map(|v| format!("\"{}\"", v))
            .join(",");
        format!(
            r"
local entities = game.surfaces[1].find_entities_filtered{{ 
    area = {area}, 
    name = {{ {filters} }} 
}}
for _, entity in ipairs(entities) do
    entity.destroy()
end
        "
        )
        .trim()
        .replace('\n', "")
    }

    fn destroy_everything(&self) -> String {
        debug!("destroying everything {:?}", self);
        let area = &self.area;
        format!(
            r"
local entities = game.surfaces[1].find_entities({area})
for _, entity in ipairs(entities) do
    entity.destroy()
end
        "
        )
        .trim()
        .replace('\n', "")
    }
}
