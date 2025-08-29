use crate::admiral::err::AdmiralResult;
use crate::admiral::lua_command::LuaCommand;
use crate::admiral::lua_command::fac_render_destroy::FacRenderDestroy;
use crate::admiral::lua_command::fac_render_text::FacRenderText;
use crate::blueprint::bpfac::position::FacBpPosition;
use crate::blueprint::output::FacItemOutput;
use crate::common::varea::VArea;
use crate::common::vpoint::{VPOINT_ZERO, VPoint};
use crate::game_entities::belt::FacEntBeltType;
use crate::game_entities::belt_transport::FacEntBeltTransport;
use crate::game_entities::direction::FacDirectionQuarter;
use std::rc::Rc;

pub fn make_render_text(output: Rc<FacItemOutput>) -> AdmiralResult<()> {
    output.admiral_execute_command(
        FacRenderDestroy::destroy_area(VArea::from_radius(VPOINT_ZERO, 10)).into_boxed(),
    )?;

    output.writei(
        FacEntBeltTransport::new(FacEntBeltType::Basic, FacDirectionQuarter::North),
        VPoint::new(5, 0),
    );

    let sep = 0.25;

    output.admiral_execute_command(
        FacRenderText::text(
            "this_is_test", //
            FacBpPosition::new(5.0, 0.0 + (sep * 0.0)),
        )
        .into_boxed(),
    )?;
    output.admiral_execute_command(
        FacRenderText::text(
            "this is string", //
            FacBpPosition::new(5.0, 0.0 + (sep * 1.0)),
        )
        .into_boxed(),
    )?;
    output.admiral_execute_command(
        FacRenderText::text(
            "other", //
            FacBpPosition::new(5.0, 0.0 + (sep * 2.0)),
        )
        .into_boxed(),
    )?;

    Ok(())
}
