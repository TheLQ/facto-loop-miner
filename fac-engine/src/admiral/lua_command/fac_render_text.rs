use crate::admiral::lua_command::LuaCommand;
use crate::admiral::lua_command::lua_syntax::{LuaSyntax, SyntaxArg};
use crate::blueprint::bpfac::position::FacBpPosition;
use crate::common::vpoint::PSugar;

#[derive(Debug)]
pub struct FacRenderText {
    pub text: String,
    pub color: Option<[u8; 3]>,
    pub pos: FacBpPosition,
}

impl LuaCommand for FacRenderText {
    fn make_lua(&self) -> String {
        assert!(!self.text.is_empty());

        let PSugar { x, y } = self.pos.sugar();
        let text = &self.text;
        let [r, g, b] = self.color.unwrap_or([1, 1, 1]);
        LuaSyntax::method("rendering.draw_text")
            .arg("surface", "game.surfaces[1]")
            .arg("target", format!("{{ {x}, {y} }}"))
            .arg("text", format!(r#""{text}""#))
            .arg("color", format!("{{ r={r},g={g},b={b} }}"))
            .build()
    }
}
