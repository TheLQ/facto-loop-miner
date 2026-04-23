use crate::state::tuneables::PathCommonTunables;
use facto_loop_miner_fac_engine::common::vpoint::VPoint;
use facto_loop_miner_fac_engine::game_entities::direction::FacDirectionQuarter;

pub fn move_further_by_section(
    radioactive: VPoint,
    input: VPoint,
    is_vertical: bool,
    tunables: &PathCommonTunables,
) -> VPoint {
    let mine_axis = if is_vertical {
        FacDirectionQuarter::North
    } else {
        FacDirectionQuarter::East
    };

    let offset_destination_pos =
        input.move_direction_int(mine_axis, tunables.base_source_section_step);
    let distance_pos = offset_destination_pos.distance_to(&radioactive);

    let offset_destination_neg =
        input.move_direction_int(mine_axis, -tunables.base_source_section_step);
    let distance_neg = offset_destination_neg.distance_to(&radioactive);

    if distance_pos > distance_neg {
        offset_destination_pos
    } else {
        offset_destination_neg
    }
}
