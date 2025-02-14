use crate::navigator::mori_cost::MoriCostMode;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Tunables {
    pub crop: CropTunables,
    pub mori: MoriTunables,
}

impl Tunables {
    pub fn new() -> Self {
        Self {
            crop: CropTunables::new(),
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
