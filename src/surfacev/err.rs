use crate::surfacev::vpoint::VPoint;
use itertools::Itertools;
use opencv::core::{Point, Point2f};
use thiserror::Error;

pub type VResult<R> = Result<R, VError>;

#[derive(Error, Debug)]
pub enum VError {
    #[error("XYOutOfBounds positions {}", positions_to_strings(positions))]
    XYOutOfBounds { positions: Vec<VPoint> },
    #[error("XYNotInteger point {}", position_to_strings_f32(position))]
    XYNotInteger { position: Point2f },
}

fn positions_to_strings(positions: &[VPoint]) -> String {
    positions.iter().map(|e| format!("{:?}", e)).join(",")
}

fn position_to_strings(position: &VPoint) -> String {
    format!("{:?}", position)
}

fn position_to_strings_f32(position: &Point2f) -> String {
    format!("{:?}", position)
}
