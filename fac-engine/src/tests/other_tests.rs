use crate::admiral::lua_command::LuaCommand;
use crate::admiral::lua_command::fac_destroy::FacDestroy;
use crate::admiral::lua_command::fac_render_destroy::FacRenderDestroy;
use crate::blueprint::output;
use crate::common::names::FacEntityName;
use crate::common::varea::VArea;
use crate::common::vpoint::VPoint;
use crate::game_entities::belt::FacEntBeltType;
use crate::game_entities::belt_transport::FacEntBeltTransport;
use crate::game_entities::direction::FacDirectionQuarter;
use crate::{
    blueprint::output::FacItemOutput,
    common::vpoint::VPOINT_TEN,
    game_blocks::{block::FacBlock, solar_bath::FacBlkSolarBath},
};
use facto_loop_miner_common::util::always_true_test;
use std::rc::Rc;

pub fn make_solar_bath_test(output: Rc<FacItemOutput>) {
    let block = FacBlkSolarBath {
        width: 3,
        height: 3,
        output,
    };
    block.generate(VPOINT_TEN);
}

pub fn max_command_size_finder(output: Rc<FacItemOutput>) {
    for test_size in 25..999 {
        let test_offset = test_size * 6;
        let top_left = VPoint::new(20, 20);
        let bottom_right = top_left.move_xy(test_offset, test_offset);
        let work_area = VArea::from_arbitrary_points_pair(top_left, bottom_right);

        output
            .admiral_execute_command(
                FacDestroy::new_filtered_area(
                    work_area.clone(),
                    vec![FacEntityName::BeltTransport(FacEntBeltType::Basic).to_fac_name()],
                )
                .into_boxed(),
            )
            .unwrap();
        output
            .admiral_execute_command(FacRenderDestroy::destroy_area(work_area).into_boxed())
            .unwrap();

        for cell_x in 0..test_offset {
            for cell_y in 0..test_offset {
                output.writei(
                    FacEntBeltTransport::new(FacEntBeltType::Basic, FacDirectionQuarter::North),
                    top_left.move_xy(cell_x, cell_y),
                )
            }
        }

        output.flush();
        if always_true_test() {
            break;
        }
    }
}
