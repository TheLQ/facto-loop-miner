use crate::navigator::mori_cost::MoriCostMode;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Tunables {
    mori: MoriTunables,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MoriTunables {
    pub cost_mode: MoriCostMode,
    pub straight_cost_unit: f32,
    pub turn_cost_unit: f32,
    pub multi_turn_lookback: usize,
    pub multi_turn_cost_unit: f32,
    pub direction_cost_unit: f32,
    pub axis_cost_unit: f32,
}

impl Default for MoriTunables {
    fn default() -> Self {
        Self {
            cost_mode: MoriCostMode::Complete,
            straight_cost_unit: 1.0,
            turn_cost_unit: 4.0,
            multi_turn_lookback: 10,
            multi_turn_cost_unit: 48.0,
            direction_cost_unit: 10.0,
            axis_cost_unit: 5.0,
        }
    }
}
