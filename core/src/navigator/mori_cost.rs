use crate::state::tuneables::MoriTunables;
use facto_loop_miner_fac_engine::common::vpoint_direction::{VPointDirectionQ, VSegment};
use facto_loop_miner_fac_engine::game_blocks::rail_hope::RailHopeLink;
use facto_loop_miner_fac_engine::game_blocks::rail_hope_single::HopeLinkType;
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
    next: &impl RailHopeLink,
    segment_points: &VSegment,
    tune: &MoriTunables,
) -> u32 {
    let result = match tune.cost_mode {
        MoriCostMode::Dummy => 5,
        MoriCostMode::DistanceManhattanOnly => {
            distance_by_basic_manhattan(next, &segment_points.end)
        }
        MoriCostMode::Complete => {
            let cost = distance_by_punish_turns(next, &segment_points.end, tune);
            let bias = axis_bias(next, tune);
            //cost
            cost + ((cost as f32 * bias) as u32)
        } // MoriCostMode::Complete => into_end_landing_bias(
          //     next,
          //     start,
          //     end,
          //     distance_by_punish_turns(parents_compare, next, end),
          // ),
    };
    result

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

fn distance_by_basic_manhattan(next: &impl RailHopeLink, end: &VPointDirectionQ) -> u32 {
    next.pos_next().distance_to(&end.0)
}

fn distance_by_punish_turns(
    next: &impl RailHopeLink,
    end: &VPointDirectionQ,
    tune: &MoriTunables,
) -> u32 {
    let base_distance = distance_by_basic_manhattan(next, end);

    let link_cost: u32 = match next.link_type() {
        HopeLinkType::Straight { .. } => tune.straight_cost_unit,
        HopeLinkType::Turn90 { .. } => tune.turn_cost_unit,
        HopeLinkType::Shift45 { .. } => todo!("shift45"),
    };

    // let num_recent_turns: u32 = parents
    //     .iter()
    //     .map(|link| match link.rtype {
    //         HopeLinkType::Turn90 { .. } => 1,
    //         _ => 0,
    //     })
    //     .sum();
    // let turn_punish = num_recent_turns * tune.multi_turn_cost_unit;

    base_distance * link_cost //+ turn_punish
}

fn axis_bias(next: &impl RailHopeLink, tune: &MoriTunables) -> f32 {
    let y_abs = next.pos_next().y().abs();
    let total = tune.crop_radius;
    let percent = y_abs as f32 / total as f32;
    percent
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
