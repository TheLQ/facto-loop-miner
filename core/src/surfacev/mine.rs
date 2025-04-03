use facto_loop_miner_common::LOCALE;
use facto_loop_miner_fac_engine::common::varea::VArea;
use facto_loop_miner_fac_engine::common::vpoint::VPoint;
use facto_loop_miner_fac_engine::game_blocks::rail_hope::RailHopeLink;
use facto_loop_miner_fac_engine::game_blocks::rail_hope_single::HopeLink;
use itertools::Itertools;
use num_format::ToFormattedString;
use serde::{Deserialize, Serialize};
use tracing::warn;

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct MinePath {
    pub mine_base: MineLocation,
    pub links: Vec<HopeLink>,
    pub cost: u32,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug, PartialOrd, Ord, Serialize, Deserialize)]
pub struct MineLocation {
    pub patch_indexes: Vec<usize>,
    pub area: VArea,
}

impl MinePath {
    pub fn total_area(&self) -> Vec<VPoint> {
        let mut new_points: Vec<VPoint> = self.links.iter().flat_map(|v| v.area()).collect_vec();

        let old_len = new_points.len();
        new_points.sort();
        new_points.dedup();
        let new_len = new_points.len();
        if old_len != new_len {
            warn!(
                "dedupe mine path from {} to {}",
                old_len.to_formatted_string(&LOCALE),
                new_len.to_formatted_string(&LOCALE)
            )
        }
        new_points
    }
}
