use crate::navigator::devo::{Rail, RailPoint};
use crate::navigator::resource_cloud::ResourceCloud;
use crate::PixelKdTree;
use kiddo::distance::squared_euclidean;
use kiddo::float::neighbour::Neighbour;

const DIRECTION_BIAS_EFFECT: f32 = 1f32;
const TURN_BIAS_EFFECT: f32 = 5f32;
const RESOURCE_BIAS_EFFECT: f32 = 20f32;

pub enum RailAction {
    TurnLeft,
    TurnRight,
    Straight,
}

pub fn calculate_bias_for_point(
    action: RailAction,
    start: &Rail,
    next: &Rail,
    end: &Rail,
    resource_cloud: &ResourceCloud,
) -> u32 {
    let cost_unit = next.distance(end) as f32;

    // Encourage going in the direction of origin.
    let direction_bias = if start.direction == end.direction {
        cost_unit * DIRECTION_BIAS_EFFECT
    } else {
        0f32
    };

    // Avoid resource patches
    let closest_resources: Vec<Neighbour<f32, usize>> = resource_cloud.kdtree.within_unsorted(
        &[start.endpoint.x as f32, start.endpoint.y as f32],
        1000f32,
        &squared_euclidean,
    );
    let mut resource_distance_bias =
        cost_unit * closest_resources.len() as f32 * RESOURCE_BIAS_EFFECT;
    // except too strong when at start
    if next.distance(start) < 500 {
        resource_distance_bias = 0f32;
    }

    let turn_bias = match action {
        RailAction::TurnLeft | RailAction::TurnRight => cost_unit * TURN_BIAS_EFFECT,
        RailAction::Straight => 0f32,
    };

    let total_cost = direction_bias + resource_distance_bias + turn_bias;
    total_cost as u32
}
