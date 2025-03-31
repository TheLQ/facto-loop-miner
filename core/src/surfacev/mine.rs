use facto_loop_miner_fac_engine::common::varea::VArea;
use facto_loop_miner_fac_engine::game_blocks::rail_hope_single::HopeLink;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
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
