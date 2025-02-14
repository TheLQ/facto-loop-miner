use crate::navigator::mori_cost::MoriCostMode;
use crate::TILES_PER_CHUNK;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Tunables {
    pub crop: CropTunables,
    pub base: BaseTunables,
    pub mori: MoriTunables,
}

impl Tunables {
    pub fn new() -> Self {
        Self {
            crop: CropTunables::new(),
            base: BaseTunables::new(),
            mori: MoriTunables::new(),
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
            resource_clear_chunks: ChunkValue(15),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MoriTunables {
    pub straight_section_size: usize,
    pub cost_mode: MoriCostMode,
    pub straight_cost_unit: f32,
    pub turn_cost_unit: f32,
    pub multi_turn_lookback: usize,
    pub multi_turn_cost_unit: f32,
    pub direction_cost_unit: f32,
    pub axis_cost_unit: f32,
}

impl MoriTunables {
    fn new() -> Self {
        Self {
            straight_section_size: 1,
            cost_mode: MoriCostMode::Complete,
            straight_cost_unit: 1.0,
            turn_cost_unit: 32.0,
            multi_turn_lookback: usize::MAX,
            // todo: turn cost unit might be better
            multi_turn_cost_unit: 0.0,
            direction_cost_unit: 10.0,
            axis_cost_unit: 5.0,
        }
    }
}

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
