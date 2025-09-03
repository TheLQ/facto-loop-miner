use crate::admiral::lua_command::LuaCommand;
use crate::admiral::lua_command::lua_syntax::{LuaSyntax, SyntaxArg};
use crate::blueprint::bpfac::position::FacBpPosition;
use crate::common::vpoint::PSugar;

#[derive(Debug)]
pub struct FacRenderText {
    text: String,
    pos: FacBpPosition,
    color: Option<[u8; 3]>,
    scale: Option<f32>,
}

impl FacRenderText {
    pub fn text(input: impl Into<String>, pos: FacBpPosition) -> Self {
        let text = input.into();
        assert!(!text.is_empty());
        Self {
            text,
            pos,
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

impl LuaCommand for FacRenderText {
    fn make_lua(&self) -> String {
        let text = &self.text;
        let [r, g, b] = self.color.unwrap_or([1, 1, 1]);
        LuaSyntax::method("rendering.draw_text")
            .arg("surface", "game.surfaces[1]")
            .arg_pos("target", self.pos)
            .arg_string("text", text)
            .arg("color", format!("{{ r={r},g={g},b={b} }}"))
            .arg_maybe("scale", self.scale, |v| v.to_string())
            .build()
    }
}
