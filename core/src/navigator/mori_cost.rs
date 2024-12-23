use crate::navigator::mori::{Rail, RailMode, RAIL_STEP_SIZE};
use crate::navigator::rail_point_compare::RailPointCompare;
use crate::navigator::resource_cloud::ResourceCloud;
use facto_loop_miner_fac_engine::common::vpoint::VPoint;
use strum::IntoStaticStr;

// const ANTI_WRONG_BIAS_EFFECT: f32 = 10f32;
// const RESOURCE_BIAS_EFFECT: f32 = 20f32;

pub const STRAIGHT_COST_UNIT: f32 = 1.0;
pub const TURN_COST_UNIT: f32 = 4.0;
// const MULTI_TURN_LOOKBACK: usize = 10;
pub const MULTI_TURN_COST_UNIT: f32 = 48.0;

pub const DIRECTION_COST_UNIT: f32 = 10.0;
pub const AXIS_COST_UNIT: f32 = 5.0;

#[derive(IntoStaticStr)]
pub enum MoriCostMode {
    Dummy,
    DistanceManhattanOnly,
    Complete,
}
pub const MORI_COST_MODE: MoriCostMode = MoriCostMode::Complete;

pub enum RailAction {
    TurnLeft,
    TurnRight,
    Straight,
}

pub fn calculate_cost_for_point(
    next: &Rail,
    start: &Rail,
    end: &VPoint,
    resource_cloud: &ResourceCloud,
    parents_compare: &[RailPointCompare],
) -> u32 {
    let result = match MORI_COST_MODE {
        MoriCostMode::Dummy => 5.0,
        MoriCostMode::DistanceManhattanOnly => distance_basic_manhattan(next, end),
        MoriCostMode::Complete => into_end_landing_bias(
            next,
            start,
            end,
            distance_with_less_parent_turns(parents_compare, next, end),
        ),
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

fn distance_basic_manhattan(next: &Rail, end: &VPoint) -> f32 {
    next.endpoint.distance_to(end) as f32
}

fn distance_with_less_parent_turns(
    parents_compare: &[RailPointCompare],
    next: &Rail,
    end: &VPoint,
) -> f32 {
    // turning is costly
    let mut total_cost = distance_basic_manhattan(next, end) / RAIL_STEP_SIZE as f32;

    total_cost += match next.mode {
        RailMode::Straight => STRAIGHT_COST_UNIT,
        RailMode::Turn(_) => TURN_COST_UNIT,
    };

    // add extra cost for previous turns
    let mut last_turns = 0u32;
    for parent in parents_compare.iter().rev().take(5) {
        if let RailMode::Turn(_) = parent.inner.mode {
            last_turns += 25;
            // TODO: If this is too low, direction bias takes over and forces rail all the way to the end
            // .pow(last_turns.min(3))
            total_cost += last_turns as f32 * MULTI_TURN_COST_UNIT;
        } else {
            last_turns = last_turns.saturating_sub(1);
        }
    }
    // let mut parent_iter = parents.iter().rev();
    // for _ in 0..MULTI_TURN_LOOKBACK {
    //     if let Some(rail) = parent_iter.next() {
    //         if let RailMode::Turn(_) = rail.mode {
    //             total_cost += MULTI_TURN_COST_UNIT;
    //         }
    //     }
    // }
    total_cost
}

fn into_end_landing_bias(next: &Rail, start: &Rail, end: &VPoint, base_distance: f32) -> f32 {
    // const BIAS_DISTANCE_START: f32 = 30.0;
    // const DIRECTION_COST_UNIT: f32 = 5.0;
    // const AXIS_COST_UNIT: f32 = 5.0;

    let mut total_cost = base_distance;

    // if next.endpoint.distance_to(&end.endpoint) > BIAS_DISTANCE_START as u32 {
    //     return 0.0;
    // }

    // Add cost if wrong direction near base
    // - Don't "hug" the base border and turn right before, overwriting many destinations
    // - Don't go behind the destination
    let direction_bias = if next.direction.is_same_axis(&start.direction) {
        0.0
    } else {
        // DIRECTION_COST_UNIT * (BIAS_DISTANCE_START - base_distance)
        DIRECTION_COST_UNIT
    };
    // total_cost += direction_bias;

    let axis_distance = start
        .distance_between_perpendicular_axis(&next.endpoint)
        .unsigned_abs() as f32
        * AXIS_COST_UNIT;
    // let axis_distance = (axis_distance - 3).max(1) as f32;
    // let axis_bias = axis_distance as f32 * AXIS_COST_UNIT;
    // (base_distance + direction_bias) * (axis_distance)
    // base_distance + (axis_distance * AXIS_COST_UNIT)

    total_cost += axis_distance;

    total_cost
}
