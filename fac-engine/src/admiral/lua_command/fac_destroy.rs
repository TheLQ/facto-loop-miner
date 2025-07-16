use crate::admiral::lua_command::LuaCommand;
use crate::common::varea::{VArea, VAreaSugar};
use itertools::Itertools;
use std::borrow::Borrow;
use tracing::debug;

#[derive(Debug)]
pub struct FacDestroy {
    area: String,
    entity_names: Vec<String>,
    is_tiles: bool,
}

impl FacDestroy {
    pub fn new_everything(radius: u32) -> Self {
        Self {
            area: format!("{{ {{ -{radius}, -{radius} }} , {{ {radius}, {radius} }} }}"),
            entity_names: Vec::new(),
            is_tiles: false,
        }
    }

    pub fn new_filtered(radius: u32, entity_names: Vec<String>) -> Self {
        if entity_names.is_empty() {
            panic!("empty entities, not destroying everything")
        }
        Self {
            area: format!("{{ {{ -{radius}, -{radius} }} , {{ {radius}, {radius} }} }}"),
            entity_names,
            is_tiles: false,
        }
    }

    pub fn new_filtered_area(area: impl Borrow<VArea>, entity_names: Vec<String>) -> Self {
        if entity_names.is_empty() {
            panic!("empty entities, not destroying everything")
        }
        let area = area.borrow();
        let VAreaSugar {
            start_x,
            start_y,
            end_x,
            end_y,
        } = area.desugar();
        Self {
            area: format!("{{ {{ {start_x}, {start_y} }} , {{ {end_x}, {end_y} }} }}"),
            entity_names,
            is_tiles: false,
        }
    }

    pub fn into_tiles(mut self) -> Self {
        self.is_tiles = true;
        self
    }
}

impl LuaCommand for FacDestroy {
    fn make_lua(&self) -> String {
        if self.entity_names.is_empty() {
            self.destroy_everything()
        } else if self.is_tiles {
            self.destroy_filtered_tiles()
        } else {
            self.destroy_filtered_entities()
        }
    }
}

impl FacDestroy {
    fn destroy_filtered_entities(&self) -> String {
        debug!("destroying filtered entities {:?}", self);
        let area = &self.area;
        let filters = self
            .entity_names
            .iter()
            .map(|v| format!("\"{v}\""))
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

    fn destroy_filtered_tiles(&self) -> String {
        debug!("destroying filtered tiles {:?}", self);
        // game.players[1].teleport({{ 1000, 1000 }})
        // rcon.print('destroy_' .. entity.name )
        // for entity in
        let area = &self.area;
        let filters = self
            .entity_names
            .iter()
            .map(|v| format!("\"{v}\""))
            .join(",");
        format!(
            r"
local tiles = game.surfaces[1].find_tiles_filtered{{ 
    area = {area}, 
    name = {{ {filters} }} 
}}
for _, tile in ipairs(tiles) do
    game.surfaces[1].set_tiles( {{ {{ name = tile.hidden_tile, position = tile.position }} }} )
end"
        )
        .trim()
        .replace('\n', "")
    }

    fn destroy_everything(&self) -> String {
        if self.is_tiles {
            panic!("tiles unsupported");
        }

        debug!("destroying everything {:?}", self);
        let area = &self.area;
        format!(
            r"
local entities = game.surfaces[1].find_entities({area})
for _, entity in ipairs(entities) do
    entity.destroy()
end"
        )
        .trim()
        .replace('\n', "")
    }
}
