use serde::{Deserialize, Serialize};

use crate::{
    admiral::lua_command::fac_surface_create_tile::FacSurfaceCreateLua,
    common::{names_tile::FacTileConcreteType, vpoint::VPoint},
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub struct FacBpTile {
    pub name: FacTileConcreteType,
    pub position: VPoint,
}

impl FacBpTile {
    pub fn new(name: FacTileConcreteType, position: VPoint) -> Self {
        Self { name, position }
    }

    pub fn to_lua(&self) -> FacSurfaceCreateLua {
        FacSurfaceCreateLua::new().with_entry(self.name.to_fac_name(), self.position)
    }
}
