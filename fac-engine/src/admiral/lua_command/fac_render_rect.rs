use crate::admiral::lua_command::LuaCommand;
use crate::admiral::lua_command::lua_syntax::{LuaSyntax, SyntaxArg};
use crate::blueprint::bpfac::position::FacBpPosition;
use crate::common::vpoint::PSugar;

#[derive(Debug)]
pub struct FacRenderRect {
    top_left: FacBpPosition,
    bottom_right: FacBpPosition,
    color: Option<[u8; 3]>,
    scale: Option<f32>,
}

impl FacRenderRect {
    pub fn rectangle(top_left: FacBpPosition, bottom_right: FacBpPosition) -> Self {
        Self {
            top_left,
            bottom_right,
            color: None,
            scale: None,
        }
    }

    pub fn with_color(mut self, color: [u8; 3]) -> Self {
        self.color = Some(color);
        self
    }

    pub fn with_scale(mut self, scale: f32) -> Self {
        self.scale = Some(scale);
        self
    }
}

impl LuaCommand for FacRenderRect {
    fn make_lua(&self) -> String {
        LuaSyntax::method("rendering.draw_rectangle")
            .arg_pos("left_top", self.top_left)
            .arg_pos("right_bottom", self.bottom_right)
            .arg("surface", "game.surfaces[1]")
            .arg_color("color", self.color.unwrap_or([1, 1, 1]))
            .build()
    }
}
