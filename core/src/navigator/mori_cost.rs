use crate::navigator::mori::PathSegmentPoints;
use crate::state::tuneables::MoriTunables;
use facto_loop_miner_fac_engine::common::vpoint_direction::VPointDirectionQ;
use facto_loop_miner_fac_engine::game_blocks::rail_hope_single::{HopeLink, HopeLinkType};
use serde::{Deserialize, Serialize};
// const ANTI_WRONG_BIAS_EFFECT: f32 = 10f32;
// const RESOURCE_BIAS_EFFECT: f32 = 20f32;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MoriCostMode {
    Dummy,
    DistanceManhattanOnly,
    Complete,
}

pub fn calculate_cost_for_link(
    next: &HopeLink,
    segment_points: &PathSegmentPoints,
    parents: &[&HopeLink],
    tune: &MoriTunables,
) -> u32 {
    let result = match tune.cost_mode {
        MoriCostMode::Dummy => 5.0,
        MoriCostMode::DistanceManhattanOnly => {
            distance_by_basic_manhattan(next, &segment_points.end)
        }
        MoriCostMode::Complete => {
            distance_by_punish_turns(parents, next, &segment_points.end, tune)
        } // MoriCostMode::Complete => into_end_landing_bias(
          //     next,
          //     start,
          //     end,
          //     distance_by_punish_turns(parents_compare, next, end),
          // ),
    };
    result as u32

    // // block it closer to base
    // let anti_wrong = if distance < 400.0 {
    //     if start.direction != end.direction {
    //         0f32
    //     } else {
    //         let v = cost_unit * ANTI_WRONG_BIAS_EFFECT;
    //         v
    //     }
    // } else {
    //     0f32
    // };

    // Avoid resource patches
    // : Vec<Neighbour<f32, usize>>
    // let closest_resources = resource_cloud
    //     .kdtree
    //     .within_unsorted::<SquaredEuclidean>(&start.endpoint.to_slice_f32(), 1000f32);
    // let mut resource_distance_bias =
    //     cost_unit * closest_resources.len() as f32 * RESOURCE_BIAS_EFFECT;
    // // except too strong when at start
    // if next.distance_to(start) < 500 {
    //     resource_distance_bias = 0f32;
    // }
    //
}

fn distance_by_basic_manhattan(next: &HopeLink, end: &VPointDirectionQ) -> f32 {
    next.next_straight_position().distance_to(&end.0) as f32
}

fn distance_by_punish_turns(
    parents: &[&HopeLink],
    next: &HopeLink,
    end: &VPointDirectionQ,
    tune: &MoriTunables,
) -> f32 {
    let base_distance = distance_by_basic_manhattan(next, end);

    let link_cost: f32 = match next.rtype {
        HopeLinkType::Straight { length } => tune.straight_cost_unit,
        HopeLinkType::Turn90 { .. } => tune.turn_cost_unit,
        HopeLinkType::Shift45 { .. } => todo!("shift45"),
    };

    let num_recent_turns: f32 = parents
        .iter()
        .map(|link| match link.rtype {
            HopeLinkType::Turn90 { .. } => 1.0,
            _ => 0.0,
        })
        .sum();
    let turn_punish = num_recent_turns * tune.multi_turn_cost_unit;

    (base_distance * link_cost) + turn_punish
}

// fn into_end_landing_bias(next: &Rail, start: &Rail, end: &VPoint, base_distance: f32) -> f32 {
//     // const BIAS_DISTANCE_START: f32 = 30.0;
//     // const DIRECTION_COST_UNIT: f32 = 5.0;
//     // const AXIS_COST_UNIT: f32 = 5.0;
//
//     let mut total_cost = base_distance;
//
//     // if next.endpoint.distance_to(&end.endpoint) > BIAS_DISTANCE_START as u32 {
//     //     return 0.0;
//     // }
//
//     // Add cost if wrong direction near base
//     // - Don't "hug" the base border and turn right before, overwriting many destinations
//     // - Don't go behind the destination
//     let direction_bias = if next.direction.is_same_axis(&start.direction) {
//         0.0
//     } else {
//         // DIRECTION_COST_UNIT * (BIAS_DISTANCE_START - base_distance)
//         DIRECTION_COST_UNIT
//     };
//     // total_cost += direction_bias;
//
//     let axis_distance = start
//         .distance_between_perpendicular_axis(&next.endpoint)
//         .unsigned_abs() as f32
//         * AXIS_COST_UNIT;
//     // let axis_distance = (axis_distance - 3).max(1) as f32;
//     // let axis_bias = axis_distance as f32 * AXIS_COST_UNIT;
//     // (base_distance + direction_bias) * (axis_distance)
//     // base_distance + (axis_distance * AXIS_COST_UNIT)
//
//     total_cost += axis_distance;
//
//     total_cost
// }
