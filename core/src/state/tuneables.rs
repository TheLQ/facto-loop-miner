use crate::TILES_PER_CHUNK;
use crate::navigator::MoriCostMode;
use facto_loop_miner_fac_engine::game_blocks::rail_hope_single::SECTION_POINTS_I32;
use facto_loop_miner_fac_engine::game_entities::rail_straight::RAIL_STRAIGHT_DIAMETER_I32;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Tunables {
    pub crop: CropTunables,
    pub base: BaseTunables,
    pub path_common: PathCommonTunables,
    pub mori: MoriTunables,
    pub altare: AltareTunables,
}

impl Tunables {
    pub fn new() -> Self {
        Self {
            crop: CropTunables::new(),
            base: BaseTunables::new(),
            path_common: PathCommonTunables::new(),
            mori: MoriTunables::new(),
            altare: AltareTunables::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CropTunables {
    radius: usize,
}

impl CropTunables {
    fn new() -> Self {
        Self { radius: 1000 }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BaseTunables {
    pub base_chunks: ChunkValue,
    pub resource_clear_chunks: ChunkValue,
}

impl BaseTunables {
    fn new() -> Self {
        Self {
            base_chunks: ChunkValue(2),
            resource_clear_chunks: ChunkValue(25),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MoriTunables {
    pub straight_section_size: usize,
    pub cost_mode: MoriCostMode,
    pub straight_cost_unit: u32,
    pub turn_cost_unit: u32,
    pub multi_turn_lookback: usize,
    pub multi_turn_cost_unit: u32,
    pub direction_cost_unit: u32,
    pub axis_cost_unit: u32,
    pub crop_radius: u32,
}

impl MoriTunables {
    fn new() -> Self {
        Self {
            straight_section_size: 1,
            cost_mode: MoriCostMode::Complete,
            straight_cost_unit: 1,
            turn_cost_unit: 2,
            multi_turn_lookback: usize::MAX,
            // todo: turn cost unit might be better
            multi_turn_cost_unit: 0,
            direction_cost_unit: 10,
            axis_cost_unit: 5,
            crop_radius: 1000,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct PathCommonTunables {
    pub scan_size: i32,
    pub base_source_section_step: i32,
    pub base_source_intra_forward: i32,
    pub base_source_intra_sideways: i32,
    /// number of rails per intra
    pub base_source_intra_rails: u8,
    pub mine_further_attempts: u8,
}

impl PathCommonTunables {
    fn new() -> Self {
        Self {
            scan_size: 120,
            base_source_section_step: SECTION_POINTS_I32,
            base_source_intra_forward: (SECTION_POINTS_I32 / 2)
                // make positive
                + 1
                // rails
                + (RAIL_STRAIGHT_DIAMETER_I32 * 1),
            base_source_intra_sideways: 6,
            base_source_intra_rails: 4,
            mine_further_attempts: 2,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AltareTunables {
    pub step_size: usize,
    pub queue_redo: usize,
    pub queue_scan: usize,
}

impl AltareTunables {
    fn new() -> Self {
        Self {
            step_size: 120 * 3,
            queue_redo: 2,
            queue_scan: 2,
        }
    }
}

/// at 3000 crop
/// - 20 generates mostly 1, 2, some 3
/// - 40 generates slightly more 3
/// - 80 generates way less 1, more 2, good 3,4
/// - 160 generates mostly 3 - very good
/// - 220 generates 10 batch, too big
// pub const PERPENDICULAR_SCAN_WIDTH: i32 = 120;

/// A Factorio chunk
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(transparent)]
#[repr(transparent)]
pub struct ChunkValue(usize);

impl ChunkValue {
    pub fn as_tiles(&self) -> usize {
        self.0 * TILES_PER_CHUNK
    }

    pub fn as_tiles_u32(&self) -> u32 {
        self.as_tiles() as u32
    }

    pub fn as_tiles_i32(&self) -> i32 {
        self.as_tiles() as i32
    }
}
