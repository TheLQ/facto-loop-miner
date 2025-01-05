use super::LuaCommand;
use crate::common::vpoint::VPoint;
use itertools::Itertools;

#[derive(Debug)]
pub struct FacSurfaceCreateLua {
    tiles: Vec<FacSurfaceCreateLuaEntry>,
}

#[derive(Debug)]
struct FacSurfaceCreateLuaEntry {
    pub name: String,
    pub position: VPoint,
}

impl FacSurfaceCreateLua {
    pub fn new() -> Self {
        Self { tiles: Vec::new() }
    }

    pub fn with_entry(mut self, name: String, position: VPoint) -> Self {
        self.tiles.push(FacSurfaceCreateLuaEntry { name, position });
        self
    }
}

impl LuaCommand for FacSurfaceCreateLua {
    fn make_lua(&self) -> String {
        // TODO: anti-dedupe logic?

        let tiles_lua = self
            .tiles
            .iter()
            .map(|FacSurfaceCreateLuaEntry { name, position }| {
                format!(
                    r#"{{ name="{name}", position={{ {x}, {y} }} }}"#,
                    x = position.x(),
                    y = position.y()
                )
            })
            .join(",");

        let lua = format!(
            r#"game.surfaces[1].set_tiles{{ 
            {tiles_lua}
            }}"#,
        );
        lua
    }
}
