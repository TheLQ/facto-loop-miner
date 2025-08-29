use crate::admiral::err::AdmiralResult;
use crate::admiral::lua_command::LuaCommand;
use crate::admiral::lua_command::fac_render_destroy::FacRenderDestroy;
use crate::admiral::lua_command::fac_render_text::FacRenderText;
use crate::blueprint::bpfac::position::FacBpPosition;
use crate::blueprint::output::FacItemOutput;
use crate::common::varea::VArea;
use crate::common::vpoint::VPOINT_ZERO;
use std::rc::Rc;

pub fn make_render_text(output: Rc<FacItemOutput>) -> AdmiralResult<()> {
    output.admiral_execute_command(
        FacRenderDestroy::destroy_area(VArea::from_radius(VPOINT_ZERO, 4)).into_boxed(),
    )?;

    output.admiral_execute_command(
        FacRenderText {
            text: "this_is_test".into(),
            color: None,
            pos: FacBpPosition::new(5.0, 0.0),
        }
        .into_boxed(),
    )?;
    Ok(())
}
