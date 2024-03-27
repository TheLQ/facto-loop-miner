use crate::navigator::mori::{Rail, RailMode};
use crate::navigator::resource_cloud::ResourceCloud;

const TURN_BIAS_EFFECT: f32 = 5f32;
const ANTI_WRONG_BIAS_EFFECT: f32 = 10f32;
const RESOURCE_BIAS_EFFECT: f32 = 20f32;

pub enum RailAction {
    TurnLeft,
    TurnRight,
    Straight,
}

pub fn calculate_cost_for_point(
    next: &Rail,
    end: &Rail,
    resource_cloud: &ResourceCloud,
    parents: &[Rail],
) -> u32 {
    // base distance
    let base_distance = match 2 {
        1 => distance_basic_manhattan(next, end),
        2 => distance_with_less_parent_turns(parents, next),
        _ => panic!("Asd"),
    };
    let end_landing_bias = end_landing_bias(parents, next, end, base_distance);
    (base_distance + end_landing_bias) as u32

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
    // let turn_bias = match action {
    //     RailAction::TurnLeft | RailAction::TurnRight => cost_unit * TURN_BIAS_EFFECT,
    //     RailAction::Straight => 0f32,
    // };
}

fn distance_basic_manhattan(next: &Rail, end: &Rail) -> f32 {
    next.distance_to(end) as f32
}

fn distance_with_less_parent_turns(parents: &[Rail], next: &Rail) -> f32 {
    const COST_UNIT: f32 = 6.0;
    const TURN_MULTIPLER: f32 = 2.0;
    // const MULTI_TURN_LOOKBACK: usize = 10;
    const MULTI_TURN_COST_UNIT: f32 = 4.0;

    // turning is costly
    let mut total_cost = match next.mode {
        RailMode::Straight => COST_UNIT,
        RailMode::Turn(_) => COST_UNIT * TURN_MULTIPLER,
    };

    // add extra cost for previous turns
    for parent in parents {
        if let RailMode::Turn(_) = parent.mode {
            total_cost += MULTI_TURN_COST_UNIT;
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

fn end_landing_bias(parents: &[Rail], next: &Rail, end: &Rail, base_distance: f32) -> f32 {
    const BIAS_DISTANCE_START: f32 = 30.0;
    const DIRECTION_COST_UNIT: f32 = 6.0;
    const AXIS_COST_UNIT: f32 = 6.0;

    // if next.endpoint.distance_to(&end.endpoint) > BIAS_DISTANCE_START as u32 {
    //     return 0.0;
    // }

    // Add cost if wrong direction near base
    // - Don't "hug" the base border and turn right before, eliminating many destinations
    // - Don't go behind the destination
    let direction_bias = if next.direction != end.direction {
        DIRECTION_COST_UNIT * (BIAS_DISTANCE_START - base_distance)
    } else {
        0.0
    };

    let axis_distance = end.distance_between_parallel_axis(next).abs();
    let axis_distance = (axis_distance - 6).min(0);
    let axis_bias = axis_distance as f32 * AXIS_COST_UNIT;

    direction_bias + axis_bias
}
